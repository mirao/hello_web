use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use chrono::Local;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Log with timestamp to display in raw mode
pub fn log(message: String) {
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S");

    println!("{}: {}\r", current_time, message);
}

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop
            /* Loop is needed, otherwise receiver is dropped when thread finishes and next sending may fail */
            {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        log(format!("Worker {} got a job; executing.", id));

                        job();
                    }

                    Message::Terminate => {
                        log(format!("Worker {} was told to terminate.", id));

                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

#[derive(PartialEq, Debug)]
pub enum PoolCreationError {
    ZeroSize,
}

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
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

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        log("Sending terminate message to all workers.".to_string());

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        log("Shutting down all workers.".to_string());

        for worker in &mut self.workers {
            log(format!("Shutting down worker {}", worker.id));

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
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
