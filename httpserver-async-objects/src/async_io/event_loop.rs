use std::{
    collections::HashMap,
    io::Result,
    pin::Pin,
    sync::{Arc, RwLock},
};

use colored::Colorize;

use super::event_handler::EventHandler;
use super::reactor::Reactor;

pub struct EventLoop {
    pub reactor: Arc<RwLock<Reactor>>,
    pub sources: HashMap<usize, Box<dyn EventHandler>>,
}

impl EventLoop {
    pub fn new(reactor: Arc<RwLock<Reactor>>) -> Self {
        EventLoop {
            reactor,
            sources: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            // process ready queue
            // 1. keep processing as long as we aren't waiting for I/O
            loop {
                if let Some(id) = {
                    let mut reactor_lock = self.reactor.write().unwrap();
                    reactor_lock.tasks.pop()
                } {
                    if let Some(source) = self.sources.get_mut(&id) {
                        source.poll();
                    }
                }

                // 2. handle unregister events
                self.handle_register();

                // if there are no more ready events to process, wait for I/O
                if self.reactor.read().unwrap().tasks.is_empty() {
                    break;
                }
            }

            // 3. handle unregister events
            self.handle_unregister();

            // 4. when all is done wait for I/O
            // wait for i/o
            // i/o events will requeue associated pending tasks
            self.wait_for_io()?;
        }
    }

    fn wait_for_io(&mut self) -> std::io::Result<()> {
        println!(
            "{} {:?} waiting for I/O",
            format!("[{}]", std::process::id()).truecolor(0, 255, 136),
            std::thread::current().id()
        );
        let mut events = Vec::new();
        {
            let cx = self.reactor.read().unwrap();
            cx.poller.wait(&mut events, None)?;
        }
        for ev in &events {
            if let Some(source) = self.sources.get_mut(&ev.key) {
                if source.as_ref().matches(ev) {
                    source.event(ev);
                }
            }
        }

        Ok(())
    }

    fn handle_register(&mut self) {
        let mut cx = self.reactor.write().unwrap();
        while let Some((id, source)) = cx.register.pop() {
            println!(
                "{} {:?} reactor registered source Fd({}) @ {}",
                format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                std::thread::current().id(),
                &id,
                source.as_ref().name()
            );
            self.sources.insert(id, source);
        }
    }

    fn handle_unregister(&mut self) {
        let mut cx = self.reactor.write().unwrap();
        while let Some(id) = cx.unregister.pop() {
            println!(
                "{} {:?} reactor removed source Fd({})",
                format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                std::thread::current().id(),
                &id
            );
            self.sources.remove(&id);
        }
    }
}
