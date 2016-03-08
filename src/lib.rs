use std::path::PathBuf;
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub struct ScannerBuilder {
    path: PathBuf,
    max_threads: usize,
    subscribers: Vec<Sender<Directory>>,
}

impl ScannerBuilder {

    pub fn new() -> ScannerBuilder {
        ScannerBuilder { path: PathBuf::new(), max_threads: 10, subscribers: vec![] }
    }

    pub fn start_from_path(mut self, path: &str) -> Self {
        self.path = PathBuf::from(path);
        self
    }

    pub fn max_threads(mut self, thread_limit: usize) -> Self {
        self.max_threads = thread_limit;
        self
    }

    pub fn update_subscriber(mut self, subscriber: Sender<Directory>) -> Self {
        self.subscribers.push(subscriber);
        self
    }

    pub fn build(&self) -> DirectoryScanner {
        let mut scanner = DirectoryScanner::new(self.path.clone());
        scanner.set_concurrency_limit(self.max_threads - 1);
        for subscriber in self.subscribers.iter() {
            scanner.add_subscriber(subscriber.clone());
        }
        scanner
    }
}

#[derive(Debug, Clone)]
pub struct Directory {
    files: Vec<File>,
    path: PathBuf,
    sub_directories: Vec<Directory>,
}

impl Directory {

    pub fn new(path: PathBuf) -> Self {
        Directory { files: vec![], path: path, sub_directories: vec![] }
    }

    pub fn len(&self) -> usize {
        let total = &self.sub_directories.iter()
                       .fold(self.files.len(), |acc, ref directory| acc + directory.len());
        *total
    }

    pub fn push(&mut self, filepath: String) {
        self.files.push(File::new(filepath.clone(), self.path.clone()));
    }

    pub fn extend(&mut self, other: &Directory) {
        // TODO this will make to make sure the other is not higher up the tree then self right?
        self.sub_directories.push(other.clone());
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut result = vec![];
        let mut flattened_files = vec![];
        for file in &self.files {
            flattened_files.push(file.as_string());
        }
        result.extend(flattened_files.clone());
        for directory in &self.sub_directories {
            result.extend(directory.flatten());
        }
        result
    }

}


#[derive(Debug, Clone)]
pub struct File {
    basename: String,
    dirname: PathBuf,
}

impl File {

    pub fn new(basename: String, dirname: PathBuf) -> Self {
        File { basename: basename, dirname: dirname }
    }

    pub fn path(&self) -> PathBuf {
        self.dirname.join(self.basename.clone())
    }

    pub fn file_name(&self) -> PathBuf {
        PathBuf::from(self.basename.clone())
    }

    pub fn as_string(&self) -> String {
        self.path().to_str().unwrap().to_string()
    }

}

pub struct DirectoryScanner {
    root_dir: PathBuf,
    subscribers: Vec<Arc<Mutex<Sender<Directory>>>>,
    concurrency_limit: usize,
    pub max_concurrency_reached: usize,
    pub current_concurrency: Arc<AtomicUsize>
}

impl DirectoryScanner {

    pub fn new(root_dir: PathBuf) -> DirectoryScanner {
        DirectoryScanner { root_dir: root_dir, subscribers: vec![], max_concurrency_reached: 0, concurrency_limit: 9, current_concurrency: Arc::new(AtomicUsize::new(0)) }
    }

    pub fn scan(&mut self) -> Directory {
        let mut file_system = Directory::new(self.root_dir.clone());
        match fs::read_dir(&self.root_dir) {
            Ok(read_dir) => {
                for entry in read_dir {
                    match entry {
                        Ok(entry) => {
                            let filetype = entry.file_type().unwrap();
                            if filetype.is_file() {
                                file_system.push(entry.path().file_name().unwrap().to_str().unwrap().to_string());
                            } else if filetype.is_dir() && !filetype.is_symlink() {
                                let path = PathBuf::from(entry.path().to_str().unwrap().to_string());
                                if self.concurrency_limit_reached() {
                                    let sub_filepaths = self.scan_directory(path);
                                    file_system.extend(&sub_filepaths);
                                } else {
                                    self.scan_directory_within_thread(path);
                                    // this doesn't update the return value
                                }
                            }
                        }
                        Err(_) => {  }
                    }
                }
            }
            Err(_) => { } // this should never happen what do we do just in case?
        }
        for subscriber in self.subscribers.iter() {
            subscriber.lock().unwrap().send(file_system.clone()).unwrap();
        }

        file_system
    }

    pub fn add_subscriber(&mut self, subscriber: Sender<Directory>) {
        self.subscribers.push(Arc::new(Mutex::new(subscriber)));
    }

    pub fn set_concurrency_limit(&mut self, limit: usize) {
        self.concurrency_limit = limit;
    }

    //------------- private methods -------------//

    fn scan_directory(&self, path: PathBuf) -> Directory {
        let mut sub_scanner = DirectoryScanner::new(path);
        sub_scanner.set_concurrency_limit(self.concurrency_limit);
        sub_scanner.current_concurrency = self.current_concurrency.clone();
        for subscriber in self.subscribers.iter() {
            sub_scanner.add_subscriber(subscriber.lock().unwrap().clone());
        }
        sub_scanner.scan()
    }

    fn scan_directory_within_thread(&mut self, path: PathBuf) {
        self.current_concurrency.fetch_add(1, Ordering::Relaxed);
        if self.current_concurrency.load(Ordering::Relaxed) > self.max_concurrency_reached {
          self.max_concurrency_reached =   self.current_concurrency.load(Ordering::Relaxed);
        }
        let local_path = path.clone();
        let local_current_concurrency = self.current_concurrency.clone();
        let local_subscribers = self.subscribers.clone();
        thread::spawn(move||{
            let mut scanner = DirectoryScanner::new(local_path);
            scanner.current_concurrency = local_current_concurrency.clone();
            for subscriber in local_subscribers.iter() {
                scanner.add_subscriber(subscriber.lock().unwrap().clone());
            }
            scanner.scan();
            local_current_concurrency.fetch_sub(1, Ordering::Relaxed);
        });
    }

    fn concurrency_limit_reached(&self) -> bool {
        self.current_concurrency.load(Ordering::Relaxed) >= self.concurrency_limit
    }
}
