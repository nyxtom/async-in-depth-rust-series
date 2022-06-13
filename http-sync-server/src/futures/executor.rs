use core::future::Future;
use std::{
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
};

use crate::router::BoxedFuture;

use super::worker::Worker;

pub type Job = BoxedFuture<'static, ()>;

pub enum Task {
    Schedule(Job),
    Exit,
}

pub struct Executor {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Task>,
}

impl Executor {
    pub fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        for id in 0..size {
            workers.push(Worker::new(id, tx.clone(), Arc::clone(&rx)));
        }

        Executor {
            workers,
            sender: tx,
        }
    }

    pub fn spawn(&self, f: impl Future<Output = ()> + 'static + Send + Sync) {
        let job = Box::pin(f);
        self.sender.send(Task::Schedule(job)).unwrap();
    }
}
