use std::future::Future;
use std::io::{self, Result};
use std::net;
use std::pin::Pin;
use std::task::{Context, Poll};

use colored::Colorize;

use crate::async_io::reactor::REACTOR;

use super::client::TcpClient;

pub struct TcpListener {
    listener: net::TcpListener,
}

impl TcpListener {
    pub fn bind(addr: &str) -> Result<TcpListener> {
        let listener = net::TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(TcpListener { listener })
    }

    pub fn accept(&self) -> Accept {
        REACTOR.with(|current| {
            let current = current.borrow();
            current.add(&self.listener);
        });

        Accept {
            listener: &self.listener,
        }
    }
}

pub struct Accept<'listener> {
    listener: &'listener net::TcpListener,
}

impl Future for Accept<'_> {
    type Output = Result<(TcpClient, net::SocketAddr)>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.listener.accept() {
            Ok((stream, addr)) => Poll::Ready(Ok((TcpClient::new(stream), addr))),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!(
                    "{} {:?} tcp listener 127.0.0.1:7000 accept() would block, pending future",
                    format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                    std::thread::current().id()
                );
                REACTOR.with(|current| {
                    let mut current = current.borrow_mut();
                    current.wake_on_readable(self.listener, cx);
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl Drop for Accept<'_> {
    fn drop(&mut self) {
        REACTOR.with(|current| {
            let mut current = current.borrow_mut();
            current.remove(self.listener);
        });
    }
}
