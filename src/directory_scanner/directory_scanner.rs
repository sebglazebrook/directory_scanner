use std::path::PathBuf;
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use time;

use directory_scanner::Directory;

pub struct DirectoryScanner {
    root_dir: PathBuf,
    subscribers: Vec<Arc<Mutex<Sender<Directory>>>>,
    concurrency_limit: usize,
    pub max_concurrency_reached: usize,
    pub current_concurrency: Arc<AtomicUsize>,
    pub running_scanners: Arc<AtomicUsize>,
    last_event: Arc<AtomicUsize>,

}

impl DirectoryScanner {

    pub fn new(root_dir: PathBuf, last_event: Arc<AtomicUsize>) -> DirectoryScanner {
        DirectoryScanner {
            root_dir: root_dir,
            subscribers: vec![],
            max_concurrency_reached: 0,
            concurrency_limit: 9,
            current_concurrency: Arc::new(AtomicUsize::new(0)),
            running_scanners: Arc::new(AtomicUsize::new(0)),
            last_event: last_event,
        }
    }

    pub fn scan(&mut self) -> Directory {
        self.last_event.store(time::now().to_timespec().sec as usize, Ordering::Relaxed);
        self.running_scanners.fetch_add(1, Ordering::Relaxed);
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
            // TODO enable this when multithreaded is working again
            //subscriber.lock().unwrap().send(file_system.clone()).unwrap();
        }

        self.running_scanners.fetch_sub(1, Ordering::Relaxed);
        self.last_event.store(time::now().to_timespec().sec as usize, Ordering::Relaxed);
        file_system
    }

    pub fn add_subscriber(&mut self, subscriber: Arc<Mutex<Sender<Directory>>>) {
        self.subscribers.push(subscriber);
    }

    pub fn set_concurrency_limit(&mut self, limit: usize) {
        self.concurrency_limit = limit;
    }

    pub fn complete(&self) -> bool {
        self.running_scanners.load(Ordering::Relaxed) == 0
            && self.current_concurrency.load(Ordering::Relaxed) == 0
            && ((time::now().to_timespec().sec as usize) - self.last_event.load(Ordering::Relaxed) > 1)
    }

    //------------- private methods -------------//

    fn scan_directory(&self, path: PathBuf) -> Directory {
        let mut sub_scanner = DirectoryScanner::new(path, self.last_event.clone());
        sub_scanner.set_concurrency_limit(self.concurrency_limit);
        sub_scanner.current_concurrency = self.current_concurrency.clone();
        for subscriber in self.subscribers.iter() {
            sub_scanner.add_subscriber(subscriber.clone());
        }
        sub_scanner.scan()
    }

    fn scan_directory_within_thread(&mut self, path: PathBuf) {
        self.current_concurrency.fetch_add(1, Ordering::Relaxed);
        if self.current_concurrency.load(Ordering::Relaxed) > self.max_concurrency_reached {
          self.max_concurrency_reached = self.current_concurrency.load(Ordering::Relaxed);
        }
        let local_path = path.clone();
        let local_current_concurrency = self.current_concurrency.clone();
        let local_subscribers = self.subscribers.clone();
        let running_scanners = self.running_scanners.clone();
        let last_event = self.last_event.clone();
        thread::spawn(move||{
            let mut scanner = DirectoryScanner::new(local_path, last_event);
            scanner.current_concurrency = local_current_concurrency;
            for subscriber in local_subscribers.iter() {
                scanner.add_subscriber(subscriber.clone());
            }
            scanner.running_scanners = running_scanners;
            scanner.scan();
            scanner.current_concurrency.fetch_sub(1, Ordering::Relaxed);
        });
    }

    fn concurrency_limit_reached(&self) -> bool {
        self.current_concurrency.load(Ordering::Relaxed) >= self.concurrency_limit
    }
}
