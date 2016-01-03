use std::path::PathBuf;
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub enum ResultFormat {
    Flat,
    Nested,
}

//ScannerBuilder::new().path(path).nested_results().update_subscriber(subscriber).finish();
//ScannerBuilder::new().path(path).flat_results().finish();

pub struct DirectoryScanner {
    root_dir: PathBuf,
    subscribers: Vec<Arc<Mutex<Sender<Vec<String>>>>>,
    concurrency_limit: usize,
    pub max_concurrency_reached: usize,
    pub current_concurrency: Arc<AtomicUsize>
}

impl DirectoryScanner {

    pub fn new(root_dir: PathBuf, result_format: ResultFormat) -> DirectoryScanner {
        DirectoryScanner { root_dir: root_dir, subscribers: vec![], max_concurrency_reached: 0, concurrency_limit: 9, current_concurrency: Arc::new(AtomicUsize::new(0)) }
    }

    pub fn scan(&mut self) -> Vec<String> {
        let mut filepaths = vec![];
        match fs::read_dir(&self.root_dir) {
            Ok(read_dir) => {
                for entry in read_dir {
                    match entry {
                        Ok(entry) => {
                            let filetype = entry.file_type().unwrap();
                            if filetype.is_file() {
                                filepaths.push(entry.path().to_str().unwrap().to_string());
                            } else if filetype.is_dir() && !filetype.is_symlink() {
                                let path = PathBuf::from(entry.path().to_str().unwrap().to_string());
                                if self.concurrency_limit_reached() {
                                    let sub_filepaths = self.scan_directory(path);
                                    filepaths.extend(sub_filepaths.clone());
                                } else {
                                    self.scan_directory_within_thread(path);
                                    // this means it doesn't return anything

                                    //filepaths.extend(sub_filepaths.clone());
                                }
                            }
                        }
                        Err(_) => {  }
                    }
                }
            }
            Err(_) => {} // this should never happen what do we do just in case?
        }
        for subscriber in self.subscribers.iter() {
            subscriber.lock().unwrap().send(filepaths.clone()).unwrap();
        }
        filepaths
    }

    pub fn add_subscriber(&mut self, subscriber: Sender<Vec<String>>) {
        self.subscribers.push(Arc::new(Mutex::new(subscriber)));
    }

    pub fn set_concurrency_limit(&mut self, limit: usize) {
        self.concurrency_limit = limit;
    }

    //------------- private methods -------------//

    fn scan_directory(&self, path: PathBuf) -> Vec<String> {
        let mut sub_scanner = DirectoryScanner::new(path, ResultFormat::Flat);
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
            let mut scanner = DirectoryScanner::new(local_path, ResultFormat::Flat);
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
