use std::{
    cell::RefCell,
    collections::HashMap,
    io::Result,
    task::{Context, Waker},
    time::Duration,
};

use polling::{Event, Poller, Source};

thread_local! {
    pub static REACTOR: RefCell<Reactor> = RefCell::new(Reactor::new())
}

pub struct Reactor {
    readable: HashMap<usize, Vec<Waker>>,
    writable: HashMap<usize, Vec<Waker>>,
    poller: Poller,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            readable: HashMap::new(),
            writable: HashMap::new(),
            poller: Poller::new().unwrap(),
        }
    }

    fn get_interest(&self, key: usize) -> Event {
        let readable = self.readable.contains_key(&key);
        let writable = self.writable.contains_key(&key);
        match (readable, writable) {
            (false, false) => Event::none(key),
            (true, false) => Event::readable(key),
            (false, true) => Event::writable(key),
            (true, true) => Event::all(key),
        }
    }

    pub fn wake_on_readable(&mut self, source: impl Source, cx: &mut Context) {
        let key = source.raw() as usize;
        self.readable
            .entry(key)
            .or_default()
            .push(cx.waker().clone());
        self.poller.modify(source, self.get_interest(key)).unwrap();
    }

    pub fn wake_on_writable(&mut self, source: impl Source, cx: &mut Context) {
        let fd = source.raw();

        let key = fd as usize;
        self.writable
            .entry(key)
            .or_default()
            .push(cx.waker().clone());

        self.poller.modify(source, self.get_interest(key)).unwrap();
    }

    pub fn remove(&mut self, source: impl Source) {
        let key = source.raw() as usize;
        self.poller.delete(source).unwrap();
        self.readable.remove(&key);
        self.writable.remove(&key);
    }

    pub fn add(&self, source: impl Source) {
        let key = source.raw() as usize;
        self.poller.add(source, self.get_interest(key)).unwrap();
    }

    pub fn wakers(&mut self, events: Vec<Event>) -> Vec<Waker> {
        let mut wakers = Vec::new();

        for ev in events {
            if let Some((_, readers)) = self.readable.remove_entry(&ev.key) {
                for waker in readers {
                    wakers.push(waker);
                }
            }
            if let Some((_, writers)) = self.writable.remove_entry(&ev.key) {
                for waker in writers {
                    wakers.push(waker);
                }
            }
        }

        wakers
    }

    pub fn wait(&self, events: &mut Vec<Event>, timeout: Option<Duration>) -> Result<usize> {
        self.poller.wait(events, timeout)
    }

    pub fn waiting_on_events(&self) -> bool {
        !self.readable.is_empty() || !self.writable.is_empty()
    }
}
