use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::Sender;

use directory_scanner::Directory;
use super::super::DirectoryScanner;

pub struct ScannerBuilder {
    path: PathBuf,
    max_threads: usize,
    subscribers: Vec<Arc<Mutex<Sender<Directory>>>>,
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

    pub fn update_subscriber(mut self, subscriber: Arc<Mutex<Sender<Directory>>>) -> Self {
        self.subscribers.push(subscriber);
        self
    }

    pub fn build(&self) -> DirectoryScanner {
        let mut scanner = DirectoryScanner::new(self.path.clone(), Arc::new(AtomicUsize::new(0)));
        scanner.set_concurrency_limit(self.max_threads - 1);
        for subscriber in self.subscribers.iter() {
            scanner.add_subscriber(subscriber.clone());
        }
        scanner
    }
}