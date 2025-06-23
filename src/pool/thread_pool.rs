use std::future::Future;
use std::sync::Arc;
use std::thread::{self};
use crossbeam::channel::{unbounded, Sender};
use tokio::runtime::Handle;

pub struct ThreadPool {
    sender: Sender<Job>,
    handle: Handle,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: u32, handle: Handle) -> ThreadPool {
        let (sender, rx) = unbounded();
        let receiver = Arc::new(rx);
        let mut threads = vec![];
        for _ in 0..size {
            let receiver = Arc::clone(&receiver);
            let thread = thread::spawn(move || {
                loop {
                    let Ok(job): Result<Job, _> = receiver.recv() else { return; };
                    job();
                }
            });
            threads.push(thread);
        }

        return ThreadPool {
            sender,
            handle,
        }
    }

    pub fn enqueue<F, Fut>(&self, job: F) 
    where 
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handle = self.handle.clone();
        let wrapped: Job = Box::new(move || {
            let fut = job();
            handle.block_on(fut);
        });

        if let Err(e) = self.sender.send(wrapped) {
            eprintln!("Error enqueueing job: {e}");
        }
    }
}
