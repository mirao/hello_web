use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use chrono::Local;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop
            /* Loop is needed, otherwise receiver is dropped when thread finishes and next sending may fail */
            {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!(
                    "{:}: Worker {} got a job; executing.",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    id
                );

                job();
            }
        });

        Worker { id, thread }
    }
}

#[derive(PartialEq, Debug)]
pub enum PoolCreationError {
    ZeroSize,
}

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            Err(PoolCreationError::ZeroSize)
        } else {
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            Ok(ThreadPool { workers, sender })
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::{PoolCreationError, ThreadPool};

    #[test]
    fn threadpool_new_0() {
        assert_eq!(ThreadPool::new(0).unwrap_err(), PoolCreationError::ZeroSize);
    }

    #[test]
    fn threadpool_new_1() {
        assert!(ThreadPool::new(1).is_ok());
    }
}
