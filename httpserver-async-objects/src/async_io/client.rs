use super::{event_handler::EventHandler, reactor::Reactor};
use crate::{response::Response, router::Router};
use colored::Colorize;
use polling::Event;
use std::{
    net::TcpStream,
    os::unix::prelude::AsRawFd,
    sync::{Arc, Mutex, RwLock},
    thread::JoinHandle,
    time::Duration,
};

pub struct ClientRequest {
    pub client: Option<TcpStream>,
    pub router: Arc<Router>,
    pub fd: usize,
    pub state: Arc<Mutex<Option<ClientState>>>,
    pub wait_handle: Option<JoinHandle<()>>,
    pub reactor: Arc<RwLock<Reactor>>,
}

pub enum ClientState {
    Waiting,
    ReadRequest,
    WaitingReadFile,
    ReadReponseFile(i32, String),
    WriteResponse(i32, Vec<u8>, String),
    WritingResponse,
    Close(TcpStream),
    Closed,
}

impl ClientRequest {
    pub fn new(client: TcpStream, router: Arc<Router>, reactor: Arc<RwLock<Reactor>>) -> Self {
        let fd = client.as_raw_fd() as usize;
        ClientRequest {
            client: Some(client),
            router,
            fd,
            state: Arc::new(Mutex::new(None)),
            wait_handle: None,
            reactor,
        }
    }

    fn log(&self, message: &str) {
        println!(
            "{} {:?} client {} - {}",
            format!("[{}]", std::process::id()).truecolor(0, 255, 136),
            std::thread::current().id(),
            self.name(),
            message
        );
    }

    fn update_state(&mut self, state: ClientState) {
        self.state.lock().unwrap().replace(state);
    }

    fn initialize(&mut self) {
        if let Some(client) = self.client.as_ref() {
            {
                let reactor = self.reactor.write().unwrap();
                reactor.add(client, Event::readable(self.fd));
            }
            self.update_state(ClientState::Waiting); // Pending
        } else {
            self.update_state(ClientState::Closed);
        }
    }

    fn read_request(&mut self) {
        if let Some(client) = self.client.as_ref() {
            let (code, path) = self.router.read_request(client);
            self.update_state(ClientState::ReadReponseFile(code, path)); // Ready
            let mut reactor = self.reactor.write().unwrap();
            reactor.schedule(self.fd);
        } else {
            self.update_state(ClientState::Closed);
        }
    }

    fn read_response_file(&mut self, code: i32, path: String) {
        let router = self.router.clone();
        let fd = self.fd;
        let name = self.name();
        self.log(&format!("scheduling file read {}", path));
        let state_arc = self.state.clone();
        let reactor = self.reactor.clone();
        let handle = std::thread::spawn(move || {
            let contents = router.read_response_file(path.clone());

            if "static/index.html" == &path {
                println!(
                    "{} {:?} client {} /index is requested, simulating long file read times...",
                    format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                    std::thread::current().id(),
                    name
                );
                std::thread::sleep(Duration::from_millis(5000));
            }

            state_arc
                .lock()
                .unwrap()
                .replace(ClientState::WriteResponse(code, contents, path.clone()));

            println!(
                "{} {:?} client {} file read done, wakeup! state to -> WriteResponse",
                format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                std::thread::current().id(),
                name
            );

            // wake up! (if we are waiting on I/O blocking, wake that up)
            // if not, then continue and reschedule this task
            reactor.read().unwrap().notify();
            reactor.write().unwrap().schedule(fd);
        });
        self.wait_handle = Some(handle);
        self.update_state(ClientState::WaitingReadFile); // Pending
    }

    fn write_response(&mut self, code: i32, contents: Vec<u8>, path: String) {
        if let Some(handle) = self.wait_handle.take() {
            handle.join().unwrap();
        }
        let name = self.name();
        if let Some(client) = self.client.take() {
            self.update_state(ClientState::WritingResponse); // Pending
            let fd = self.fd;
            self.log("scheduling response write thread...");
            let state_arc = self.state.clone();
            let reactor = self.reactor.clone();
            let handle = std::thread::spawn(move || {
                let mut response = Response::new(client);
                response.send_file_contents(code, &path, contents).unwrap();

                println!(
                    "{} {:?} {}, response write done, wakeup! state change -> Close",
                    format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                    std::thread::current().id(),
                    name
                );

                // set close state
                let client = response.into_inner().unwrap();
                state_arc
                    .lock()
                    .unwrap()
                    .replace(ClientState::Close(client));

                // wake up! (if we are waiting on I/O blocking, wake that up)
                // if not, then continue and reschedule this task
                reactor.read().unwrap().notify();
                reactor.write().unwrap().schedule(fd);
            });
            self.wait_handle = Some(handle);
        } else {
            self.update_state(ClientState::Closed);
        }
    }

    fn close(&mut self, client: TcpStream) {
        if let Some(handle) = self.wait_handle.take() {
            handle.join().unwrap();
        }
        self.log("request done, joining final thread and exiting\n---\n");
        self.update_state(ClientState::Closed);
        self.reactor.write().unwrap().remove(self.fd, &client);
    }
}

impl EventHandler for ClientRequest {
    fn poll(&mut self) {
        let state = self.state.lock().unwrap().take();
        match state {
            None => self.initialize(),
            Some(ClientState::ReadRequest) => self.read_request(),
            Some(ClientState::ReadReponseFile(code, path)) => self.read_response_file(code, path),
            Some(ClientState::WriteResponse(code, contents, path)) => {
                self.write_response(code, contents, path)
            }
            Some(ClientState::Close(client)) => self.close(client),
            _ => {}
        }
    }

    fn event(&mut self, event: &polling::Event) {
        let state = self.state.lock().unwrap().take();
        match state {
            Some(ClientState::Waiting) => {
                if event.readable {
                    let name = self.name();
                    println!(
                        "{} {:?} client received event! updating state to ReadRequest client {}",
                        format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                        std::thread::current().id(),
                        name
                    );
                    self.update_state(ClientState::ReadRequest);
                    let mut reactor = self.reactor.write().unwrap();
                    reactor.schedule(self.fd);
                }
            }
            Some(s) => {
                self.update_state(s);
            }
            None => {}
        }
    }

    fn matches(&self, event: &polling::Event) -> bool {
        self.fd == event.key
    }

    fn id(&self) -> usize {
        self.fd
    }

    fn name(&self) -> String {
        if let Some(client) = self.client.as_ref() {
            format!("TcpStream://{}", client.peer_addr().unwrap().to_string())
        } else {
            format!("{}", self.id())
        }
    }
}
