use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{Ordering, AtomicBool};
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

   pub fn try_recv(&self) -> Option<Directory> {
           let mut return_value = None;
           let mut done = false;
           while !done {
               match self.events.try_pop() {
                   Some(event) => {
                       return_value = Some(event);
                   },
                   None  => { done = true; }
               }
           }
          return_value
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
