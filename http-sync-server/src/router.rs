use colored::Colorize;

use crate::{node::Node, response::Response};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Result},
    net::TcpStream,
};

#[derive(PartialEq, Eq, Hash)]
pub enum Method {
    GET,
}

pub type HandlerFn = fn(Response) -> Result<()>;

pub struct Router {
    routes: HashMap<Method, Node<HandlerFn>>,
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

    pub fn route_client(&self, client: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(&client);
        let buf = reader.fill_buf()?;

        // read a single line (if one exists)
        let mut line = String::new();
        let mut line_reader = BufReader::new(buf);
        let len = line_reader.read_line(&mut line)?;

        // consume bytes read from original reader
        reader.consume(len);
        if len == 0 {
            return self.bad_request(client);
        }

        let addr = client.peer_addr()?;
        println!(
            "{} @{addr} sent",
            format!("[{}]", std::process::id()).truecolor(200, 255, 136)
        );
        println!("{}", line);

        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() < 2 {
            self.bad_request(client)
        } else {
            match (parts[0], parts[1]) {
                ("GET", path) => self.handle(Method::GET, path, client),
                _ => self.bad_request(client),
            }
        }
    }

    pub fn handle(&self, method: Method, resource: &str, client: TcpStream) -> Result<()> {
        let res = Response::new(client);
        if let Some(node) = self.routes.get(&method) {
            if let Some(handler) = node.get(resource) {
                return handler(res);
            }
        }

        // default not found
        res.sendfile(404, "static/not_found.html")
    }

    pub fn bad_request(&self, client: TcpStream) -> Result<()> {
        let res = Response::new(client);
        res.sendfile(404, "static/not_found.html")
    }
}
