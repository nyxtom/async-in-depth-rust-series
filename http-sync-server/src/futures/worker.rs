use std::{
    sync::{mpsc, Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
};

use core::future::Future;

use super::executor::Task;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

fn no_op_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        no_op_raw_waker()
    }
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn no_op_waker() -> Waker {
    unsafe { Waker::from_raw(no_op_raw_waker()) }
}

impl Worker {
    pub fn new(
        id: usize,
        sender: mpsc::Sender<Task>,
        receiver: Arc<Mutex<mpsc::Receiver<Task>>>,
    ) -> Self {
        let handle = thread::spawn(move || loop {
            let task = {
                let rx = receiver.lock().unwrap();
                rx.recv().unwrap()
            };

            match task {
                Task::Schedule(mut job) => {
                    let waker = no_op_waker();
                    let mut context = Context::from_waker(&waker);
                    match Future::poll(job.as_mut(), &mut context) {
                        Poll::Ready(r) => return r,
                        Poll::Pending => {
                            sender.send(Task::Schedule(job)).unwrap();
                        }
                    }
                }
                Task::Exit => break,
            }
        });

        Worker {
            id,
            thread: Some(handle),
        }
    }
}
