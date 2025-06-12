use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{channel, Sender};

pub struct ThreadPool<T = Job> {
    sender: Sender<Job>,
    pool: Vec<JoinHandle<T>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: u32) -> ThreadPool {
        let (sender, rx) = channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut threads = vec![];
        for _ in 0..size {
            let receiver = Arc::clone(&receiver);
            let thread = thread::spawn(move || {
                loop {
                    let job: Job = receiver.lock().unwrap().recv().unwrap();
                    job();
                }
            });
            threads.push(thread);
        }

        return ThreadPool {
            sender,
            pool: threads,
        }
    }

    pub fn enqueue(&self, job: Job) {
        match self.sender.send(job) {
            Ok(_) => {
            },
            Err(e) => {
                eprintln!("Error enqueueing thread: {:?}", e);
            }
        }
    }
}
