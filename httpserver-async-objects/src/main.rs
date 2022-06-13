#![feature(trait_alias)]

mod async_io;
mod node;
mod response;
mod router;
mod routes;

use colored::*;
use router::Router;
use std::io::Result;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

use crate::async_io::event_loop::EventLoop;
use crate::async_io::listener::AsyncTcpListener;
use crate::async_io::reactor::Reactor;

fn main() -> Result<()> {
    let port = std::env::var("PORT").unwrap_or(String::from("7000"));
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;

    let mut router = Router::new();
    routes::configure(&mut router);

    let router = Arc::new(router);
    let reactor = Arc::new(RwLock::new(Reactor::new()));
    let listener = AsyncTcpListener::new(listener, router, reactor.clone());
    reactor.write().unwrap().register(listener.fd, listener);

    println!(
        "{} server listening on 127.0.0.1:{}",
        format!("[{}]", std::process::id()).truecolor(0, 255, 136),
        port
    );

    let mut event_loop = EventLoop::new(reactor);
    event_loop.run()
}
