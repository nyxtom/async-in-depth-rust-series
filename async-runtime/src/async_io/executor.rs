use std::{
    cell::RefCell,
    future::Future,
    io::Result,
    task::{Context, Poll},
};

use super::reactor::REACTOR;
use super::task_queue::{Task, TaskQueue};
use super::waker_util::waker_fn;
use colored::Colorize;

thread_local! {
    pub static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::new())
}

pub fn block_on<F>(f: F) -> Result<()>
where
    F: Future<Output = ()> + 'static,
{
    EXECUTOR.with(|executor| -> Result<()> {
        let executor = executor.borrow();
        executor.spawn(f);
        executor.run()
    })
}

pub fn spawn<F>(f: F)
where
    F: Future<Output = ()> + 'static,
{
    EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        executor.spawn(f);
    });
}

pub struct Executor {
    pub tasks: RefCell<TaskQueue>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: RefCell::new(TaskQueue::new()),
        }
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.borrow_mut().push(Task {
            future: RefCell::new(Box::pin(f)),
        });
    }

    pub fn run(&self) -> Result<()> {
        loop {
            // process ready queue
            // keep processing as long as we aren't waiting for I/O
            loop {
                if let Some(task) = {
                    let mut tasks = self.tasks.borrow_mut();
                    tasks.pop()
                } {
                    let waker = {
                        let sender = self.tasks.borrow().sender();
                        let waker_task = task.clone();
                        waker_fn(move || {
                            // executor schedule task again
                            println!(
                                "{} {:?} waking up to requeue future polling",
                                format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                                std::thread::current().id(),
                            );
                            sender.send(waker_task.clone()).unwrap();
                        })
                    };
                    let mut context = Context::from_waker(&waker);
                    println!(
                        "{} {:?} received task, polling future...",
                        format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                        std::thread::current().id(),
                    );
                    match task.future.borrow_mut().as_mut().poll(&mut context) {
                        Poll::Ready(_) => {
                            println!(
                                "{} {:?} poll ready complete on spawned task",
                                format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                                std::thread::current().id(),
                            );
                        }
                        Poll::Pending => {}
                    };
                }

                if self.tasks.borrow().is_empty() {
                    break;
                }
            }

            // when all is done wait for I/O
            // wait for i/o
            // i/o events will requeue associated pending tasks
            if !REACTOR.with(|current| current.borrow().waiting_on_events()) {
                break Ok(());
            }

            self.wait_for_io()?;

            // IO events trigger wakers, which will generate new tasks
            self.tasks.borrow_mut().receive();
        }
    }

    fn wait_for_io(&self) -> std::io::Result<usize> {
        REACTOR.with(|current| -> Result<usize> {
            println!(
                "{} {:?} waiting for I/O",
                format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                std::thread::current().id()
            );
            let mut events = Vec::new();
            {
                let reactor = current.borrow();
                reactor.wait(&mut events, None)?;
            }

            let wakers = {
                let mut reactor = current.borrow_mut();
                reactor.wakers(events)
            };

            let len = wakers.len();
            for waker in wakers {
                waker.wake();
            }

            Ok(len)
        })
    }
}
