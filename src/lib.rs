use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// n is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the number of threads is zero.
    pub fn new(n: usize) -> ThreadPool {
        assert!(n > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(n);

        for id in 0..n {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Creates a new ThreadPool.
    ///
    /// n is the number of threads in the pool.
    pub fn build(n: usize) -> Result<ThreadPool, PoolCreationError> {
        if n > 0 {
            Ok(Self::new(n))
        } else {
            return Err(PoolCreationError(String::from("n must be greater than 0")));
        }
    }

    /// Batches a closure to be run by a worker in the ThreadPool
    ///
    /// f is the closure to be run.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting Down Worker {}...", worker.id);
            if let Some(worker) = worker.thread.take() {
                worker.join().unwrap();
            }
        }
    }
}

pub struct PoolCreationError(String);

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            match job {
                Ok(job) => {
                    println!("Worker {} received a job! Executing...", id);
                    job();
                    println!("Worker {} Finished!", id);
                }
                Err(_) => {
                    println!("Worker {} Shutting Down", id);
                    break;
                }
            };
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;
