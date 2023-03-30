use std::thread::Thread;
pub struct ThreadPool {
    threads: Vec<Thread>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut threads = Vec::with_capacity(size);

        for _ in 0..size {
            // Create and store your threads here
        }

        ThreadPool { threads }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Execute the function in a worker thread
    }
}
