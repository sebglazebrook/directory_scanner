use std::path::PathBuf;
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicBool};
use std::thread;
use time;
use gitignore;
use std::env::current_dir;
use crossbeam::sync::MsQueue;

use directory_scanner::Directory;

#[derive(Clone)]
pub struct DirectoryEventBroker {
    events: Arc<MsQueue<Directory>>,
    receiving_events: Arc<AtomicBool>,
    mutex: Arc<Mutex<bool>>,
    condvar: Arc<Condvar>,
}

impl DirectoryEventBroker {

    pub fn new() -> Self {
        DirectoryEventBroker {
            events: Arc::new(MsQueue::new()),
            receiving_events: Arc::new(AtomicBool::new(true)),
            condvar: Arc::new(Condvar::new()),
            mutex: Arc::new(Mutex::new(false)),
        }
    }

    pub fn send(&self, filter_event: Directory) {
        self.events.push(filter_event);
        self.condvar.notify_one();
    }

    pub fn close(&self) {
        self.receiving_events.store(false, Ordering::Relaxed);
        self.condvar.notify_all();
    }

    pub fn recv(&self) -> Result<Directory, &str>  {
        let mut return_event = Err("I dont't know");
        if !self.receiving_events.load(Ordering::Relaxed) {
            return return_event;
        }
        let mut done = false;
        while !done {
            let mutex_guard = self.mutex.lock().unwrap();
            let _ = self.condvar.wait(mutex_guard).unwrap();
            if !self.receiving_events.load(Ordering::Relaxed) {
                done = true;
                return Err("no longer receiving events"); //TODO send a real error type
            }
            match self.events.try_pop() {
                Some(event) =>  {
                    done = true;
                    return_event = Ok(event);
                },
                None => {}
            }
        }
        return_event
    }
}

pub struct DirectoryScanner {
    absolute_base: PathBuf,
    relative_base: PathBuf,
    pub event_broker: DirectoryEventBroker, // TODO remove this from being public
    subscribers: Vec<Arc<Mutex<Sender<Directory>>>>,
    concurrency_limit: usize,
    pub max_concurrency_reached: usize,
    pub current_concurrency: Arc<AtomicUsize>, // TODO remove this from being public
    pub running_scanners: Arc<AtomicUsize>, // TODO remove this from being public
    last_event: Arc<AtomicUsize>, // TODO rename this??
    directory: Option<Directory>,

}

impl DirectoryScanner {

    pub fn new(root_dir: PathBuf, last_event: Arc<AtomicUsize>) -> DirectoryScanner {
        let (absolute_base, relative_base) = breakdown_dir_parts(root_dir);
        DirectoryScanner {
            absolute_base: absolute_base,
            relative_base: relative_base,
            subscribers: vec![],
            max_concurrency_reached: 0,
            concurrency_limit: 9,
            current_concurrency: Arc::new(AtomicUsize::new(0)),
            running_scanners: Arc::new(AtomicUsize::new(0)),
            last_event: last_event,
            event_broker: DirectoryEventBroker::new(),
            directory: None,
        }
    }

    pub fn scan(&mut self) -> Directory {
        self.last_event.store(time::now().to_timespec().sec as usize, Ordering::Relaxed);
        self.running_scanners.fetch_add(1, Ordering::Relaxed);
        let mut file_system;
        match self.directory.clone() {
            Some(dir) => {
                file_system = Directory::new(self.relative_base.clone());
                dir.extend(&file_system) },
            None => {
                file_system = Directory::new(self.relative_base.clone());
                self.directory = Some(file_system.clone());
            }
        }
        match fs::read_dir(&self.relative_base) {
            Ok(read_dir) => {
                for entry in read_dir {
                    match entry {
                        Ok(entry) => {
                            let filetype = entry.file_type().unwrap();
                            if filetype.is_file() && !entry.file_name().to_str().unwrap().starts_with(".") && !self.is_ignored_by_git(&entry.path()) {
                                file_system.push(entry.path().file_name().unwrap().to_str().unwrap().to_string());
                            } else if filetype.is_dir() && !filetype.is_symlink() && !entry.file_name().to_str().unwrap().starts_with(".") && !self.is_ignored_by_git(&entry.path()) {
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
        self.event_broker.send(file_system.clone());

        self.running_scanners.fetch_sub(1, Ordering::Relaxed);
        self.last_event.store(time::now().to_timespec().sec as usize, Ordering::Relaxed);
        file_system
    }

    pub fn extend_directory(&mut self, directory: Directory) {
        self.directory = Some(directory);
    }

    pub fn set_concurrency_limit(&mut self, limit: usize) {
        self.concurrency_limit = limit;
    }

    pub fn is_complete(&self) -> bool {
        self.running_scanners.load(Ordering::Relaxed) == 0
            && self.current_concurrency.load(Ordering::Relaxed) == 0
            && ((time::now().to_timespec().sec as usize) - self.last_event.load(Ordering::Relaxed) > 1)
    }

    pub fn event_broker(&self) -> DirectoryEventBroker {
        self.event_broker.clone()
    }

    //------------- private methods -------------//

    fn is_ignored_by_git(&self, path: &PathBuf) -> bool {
        let gitignore_path = self.absolute_base.join(&path.parent().unwrap().strip_prefix("./").unwrap()).join(".gitignore");
        let path_to_check = self.absolute_base.join(&path.strip_prefix("./").unwrap());
        if gitignore_path.exists() {
            let file = gitignore::File::new(&gitignore_path).unwrap();
            match file.is_excluded(&path_to_check) {
                Ok(result) => { result },
                Err(error) => {
                    warn!("There was an error try to check whether to ignore path '{:?}'\nError: {:?}", path, error);
                    false
                }
            }
        } else {
            false
        }
    }

    fn scan_directory(&self, path: PathBuf) -> Directory {
        let mut sub_scanner = DirectoryScanner::new(path, self.last_event.clone());
        sub_scanner.set_concurrency_limit(self.concurrency_limit);
        sub_scanner.current_concurrency = self.current_concurrency.clone();
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
        let event_broker = self.event_broker.clone();
        let directory = self.directory.clone();
        thread::spawn(move||{
            let mut scanner = DirectoryScanner::new(local_path, last_event);
            scanner.current_concurrency = local_current_concurrency;
            scanner.running_scanners = running_scanners;
            scanner.event_broker = event_broker;
            match directory {
                Some(dir) => { scanner.extend_directory(dir); },
                None => {}
            }
            scanner.scan();
            scanner.current_concurrency.fetch_sub(1, Ordering::Relaxed);
        });
    }

    fn concurrency_limit_reached(&self) -> bool {
        self.current_concurrency.load(Ordering::Relaxed) >= self.concurrency_limit
    }
}

fn breakdown_dir_parts(dir: PathBuf) -> (PathBuf, PathBuf) {
    if dir.is_absolute() {
        (dir, PathBuf::from("./"))
    } else {
        (current_dir().unwrap(), dir)
    }
}
