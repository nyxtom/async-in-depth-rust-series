use std::collections::HashMap;
use std::future::Future;
use std::io::Result;
use std::pin::Pin;

use colored::Colorize;

use crate::async_io::task_queue::LocalBoxedFuture;
use crate::async_net::client::TcpClient;

use super::node::Node;
use super::response::Response;

#[derive(PartialEq, Eq, Hash)]
pub enum Method {
    GET,
}

pub type HandlerFn = Pin<Box<dyn Fn(TcpClient) -> LocalBoxedFuture<'static, Result<()>>>>;

pub struct Router {
    routes: HashMap<Method, Node>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn insert<F, Fut>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(TcpClient) -> Fut + 'static,
        Fut: Future<Output = Result<()>> + 'static,
    {
        let node = self.routes.entry(method).or_insert(Node::new("/"));
        node.insert(path, Box::pin(move |client| Box::pin(handler(client))));
    }

    pub async fn route_client(&mut self, mut client: TcpClient) -> Result<()> {
        let mut buffer = [0; 1024];
        let n = client.read(&mut buffer).await?;

        // read a single line (if one exists)
        let req = String::from_utf8_lossy(&buffer[0..n]);
        let mut lines = req.split('\n');
        let line = lines.next().unwrap();
        println!(
            "{} {:?} client requested\n{}",
            format!("[{}]", std::process::id()).truecolor(0, 255, 136),
            std::thread::current().id(),
            &line
        );

        // consume bytes read from original reader
        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() < 2 {
            self.bad_request(client).await
        } else {
            match (parts[0], parts[1]) {
                ("GET", path) => self.handle(Method::GET, path, client).await,
                _ => self.not_found(client).await,
            }
        }
    }

    pub async fn handle(
        &mut self,
        method: Method,
        resource: &str,
        client: TcpClient,
    ) -> Result<()> {
        if let Some(node) = self.routes.get_mut(&method) {
            if let Some(handler) = node.get(resource) {
                return handler(client).await;
            }
        }

        // default not found
        self.bad_request(client).await
    }

    pub async fn bad_request(&self, client: TcpClient) -> Result<()> {
        let mut res = Response::new(client);
        res.send_file(400, "static/_400.html").await
    }

    pub async fn not_found(&self, client: TcpClient) -> Result<()> {
        let mut res = Response::new(client);
        res.send_file(404, "static/_404.html").await
    }
}
