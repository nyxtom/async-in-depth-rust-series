use std::io::{ErrorKind, Read, Result, Write};
use std::{
    future::Future,
    net,
    pin::Pin,
    task::{Context, Poll},
};

use crate::async_io::reactor::REACTOR;

pub struct TcpClient {
    stream: net::TcpStream,
}

impl TcpClient {
    pub fn new(stream: net::TcpStream) -> Self {
        TcpClient { stream }
    }

    pub fn read<'stream, T: AsMut<[u8]>>(
        &'stream mut self,
        buf: &'stream mut T,
    ) -> ReadFuture<'stream> {
        ReadFuture {
            stream: &mut self.stream,
            buf: buf.as_mut(),
        }
    }

    pub fn write<'stream, T: AsRef<[u8]>>(
        &'stream mut self,
        buf: &'stream T,
    ) -> WriteFuture<'stream> {
        WriteFuture {
            stream: &mut self.stream,
            buf: buf.as_ref(),
        }
    }

    pub fn flush(&mut self) {
        self.stream.flush().unwrap();
    }
}

pub struct WriteFuture<'stream> {
    stream: &'stream mut net::TcpStream,
    buf: &'stream [u8],
}

impl Future for WriteFuture<'_> {
    type Output = Result<usize>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        match state.stream.write(state.buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                REACTOR.with(|current| {
                    current.borrow_mut().wake_on_writable(&*state.stream, cx);
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

pub struct ReadFuture<'stream> {
    stream: &'stream mut net::TcpStream,
    buf: &'stream mut [u8],
}

impl Future for ReadFuture<'_> {
    type Output = Result<usize>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let state = self.get_mut();
        match state.stream.read(&mut state.buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                REACTOR.with(|current| {
                    current.borrow_mut().wake_on_readable(&*state.stream, cx);
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl Drop for TcpClient {
    fn drop(&mut self) {
        REACTOR.with(|current| {
            let mut current = current.borrow_mut();
            current.remove(&self.stream);
        });
    }
}
