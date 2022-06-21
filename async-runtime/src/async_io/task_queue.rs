use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub type LocalBoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub struct TaskQueue {
    sender: Sender<Rc<Task>>,
    receiver: Receiver<Rc<Task>>,
    tasks: Vec<Rc<Task>>,
}

pub struct Task {
    pub future: RefCell<LocalBoxedFuture<'static, ()>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        TaskQueue {
            sender,
            receiver,
            tasks: Vec::new(),
        }
    }

    pub fn sender(&self) -> Sender<Rc<Task>> {
        self.sender.clone()
    }

    pub fn pop(&mut self) -> Option<Rc<Task>> {
        self.tasks.pop()
    }

    pub fn push(&mut self, runnable: Task) {
        self.tasks.push(Rc::new(runnable));
    }

    pub fn receive(&mut self) {
        for runnable in self.receiver.try_iter() {
            self.tasks.push(runnable);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}
