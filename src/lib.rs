use std::sync::{Arc, mpsc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The thread_count determines the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the thread_count is zero.
    pub fn new(thread_count: usize) -> Self {
        assert!(thread_count > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(thread_count);

        for id in 0..thread_count {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }


        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, func: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(func);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate);
        }

        println!("Shutting down all workers");

        for worker in &mut self.workers {
            self.sender.send(Message::Terminate);
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {

                    println!("Worker: {} got a job; executing;", id);

                    job();
                },
                Message::Terminate => {
                    println!("Worker {} was told to terminate", id);

                    break;
                }
            }

        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}