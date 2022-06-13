use crate::router::Router;

use super::client::ClientRequest;
use super::{event_handler::EventHandler, reactor::Reactor};
use polling::Event;
use std::sync::{Arc, RwLock};
use std::{
    net::{TcpListener, TcpStream},
    os::unix::prelude::AsRawFd,
};

pub enum ListenerState {
    Waiting,
    Accepting(TcpStream),
}

pub struct AsyncTcpListener {
    pub listener: TcpListener,
    pub router: Arc<Router>,
    pub fd: usize,
    pub state: Option<ListenerState>,
    pub reactor: Arc<RwLock<Reactor>>,
}

impl AsyncTcpListener {
    pub fn new(listener: TcpListener, router: Arc<Router>, reactor: Arc<RwLock<Reactor>>) -> Self {
        listener.set_nonblocking(true).unwrap();

        // add listener to the reactor
        let fd = listener.as_raw_fd() as usize;
        {
            let reactor = reactor.write().unwrap();
            reactor.add(&listener, Event::readable(fd));
        }

        // initialize in a waiting state
        AsyncTcpListener {
            listener,
            router,
            fd,
            state: Some(ListenerState::Waiting),
            reactor,
        }
    }
}

impl EventHandler for AsyncTcpListener {
    fn poll(&mut self) {
        match self.state.take() {
            Some(ListenerState::Accepting(client)) => {
                // modify interest in read event again
                let mut reactor = self.reactor.write().unwrap();
                reactor.modify(&self.listener, Event::readable(self.fd));

                self.state.replace(ListenerState::Waiting);
                // register new client on poller
                let client = ClientRequest::new(client, self.router.clone(), self.reactor.clone());
                reactor.register(client.fd, client);
            }
            _ => {}
        }
    }

    fn event(&mut self, event: &Event) {
        if event.readable {
            let (client, addr) = self.listener.accept().unwrap();
            self.state.replace(ListenerState::Accepting(client));
            // reschedule
            let mut reactor = self.reactor.write().unwrap();
            reactor.schedule(self.fd);
        }
    }

    fn matches(&self, event: &Event) -> bool {
        event.key == self.fd
    }

    fn name(&self) -> String {
        format!(
            "TcpListener://{}",
            self.listener.local_addr().unwrap().to_string()
        )
    }

    fn id(&self) -> usize {
        self.fd
    }
}
