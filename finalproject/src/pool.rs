use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    tx: mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "size must be > 0");
        let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();

        let rx = Arc::new(Mutex::new(rx));
        for _ in 0..size {
            let rx_clone = Arc::clone(&rx);
            thread::spawn(move || loop {
                let job = rx_clone.lock().unwrap().recv();
                match job {
                    Ok(job) => job(),
                    Err(_)  => break, // channel closed
                }
            });
        }
        ThreadPool { tx }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.send(Box::new(f)).unwrap();
    }
}
