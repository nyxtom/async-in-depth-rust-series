use colored::Colorize;

use crate::node::Node;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

#[derive(PartialEq, Eq, Hash)]
pub enum Method {
    GET,
}

pub type HandlerFn = fn() -> (i32, String);

pub struct Router {
    routes: HashMap<Method, Node>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let node = self.routes.entry(method).or_insert(Node::new("/"));
        node.insert(path, handler);
    }

    pub fn read_response_file(&self, path: String) -> Vec<u8> {
        let file = std::fs::File::open(path).unwrap();
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(file);
        reader.read_to_end(&mut buf).unwrap();
        buf
    }

    pub fn read_request(&self, client: &TcpStream) -> (i32, String) {
        let mut reader = BufReader::new(client);
        let buf = reader.fill_buf().unwrap();

        // read a single line (if one exists)
        let mut line = String::new();
        let mut line_reader = BufReader::new(buf);
        let len = line_reader.read_line(&mut line).unwrap();

        println!(
            "{} {:?} client {} requested: \n{}",
            format!("[{}]", std::process::id()).truecolor(0, 255, 136),
            std::thread::current().id(),
            client.peer_addr().unwrap().to_string(),
            &line
        );

        // consume bytes read from original reader
        reader.consume(len);
        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() < 2 {
            self.bad_request(client)
        } else {
            match (parts[0], parts[1]) {
                ("GET", path) => self.handle(Method::GET, path, client),
                _ => (404, String::from("static/not_found.html")),
            }
        }
    }

    pub fn handle(&self, method: Method, resource: &str, client: &TcpStream) -> (i32, String) {
        if let Some(node) = self.routes.get(&method) {
            if let Some(handler) = node.get(resource) {
                return handler();
            }
        }

        // default not found
        self.bad_request(client)
    }

    pub fn bad_request(&self, _client: &TcpStream) -> (i32, String) {
        (404, String::from("static/not_found.html"))
    }
}
