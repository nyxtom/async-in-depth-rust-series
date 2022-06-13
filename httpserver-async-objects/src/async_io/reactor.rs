use std::pin::Pin;

use super::event_handler::EventHandler;
use polling::{Event, Poller, Source};

pub struct Reactor {
    pub tasks: Vec<usize>,
    pub register: Vec<(usize, Box<dyn EventHandler + Send + Sync>)>,
    pub unregister: Vec<usize>,
    pub poller: Poller,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            tasks: Vec::new(),
            register: Vec::new(),
            unregister: Vec::new(),
            poller: Poller::new().unwrap(),
        }
    }

    pub fn schedule(&mut self, id: usize) {
        self.tasks.push(id);
    }

    pub fn notify(&self) {
        // ensure poller is not blocking on wait
        // now that we have something to process
        self.poller.notify().unwrap();
    }

    pub fn register<T>(&mut self, id: usize, client: T)
    where
        T: EventHandler + Send + Sync + 'static,
    {
        self.schedule(id);
        self.register.push((id, Box::new(client)));
    }

    pub fn modify(&self, source: impl Source, event: Event) {
        self.poller.modify(source, event).unwrap();
    }

    pub fn remove(&mut self, id: usize, source: impl Source) {
        self.poller.delete(source).unwrap();
        self.unregister.push(id);
        self.schedule(id);
    }

    pub fn add(&self, source: impl Source, event: Event) {
        self.poller.add(source, event).unwrap();
    }
}
