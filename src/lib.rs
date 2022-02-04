use std::thread;
use std::sync::{Arc, mpsc, Mutex};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
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
            sender
        }
    }

    pub fn execute<F>(&self, func: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(func);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>
}

impl  Worker {
   pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
       let thread = thread::spawn(move || loop {
           let job = receiver.lock().unwrap().recv().unwrap();

           println!("Worker: {} got a job; executing;", id);

           job();
       });

       Worker {
           id,
           thread
       }
   }
}