mod async_io;
mod async_net;
mod web;

use crate::async_io::executor;
use crate::async_net::listener::TcpListener;
use crate::web::router::Router;
use crate::web::routes;
use std::io::Result;

fn main() -> Result<()> {
    executor::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:7000").unwrap();
        while let Ok((client, addr)) = listener.accept().await {
            executor::spawn(async {
                let mut router = Router::new();
                routes::configure(&mut router);
                router.route_client(client).await.unwrap();
            });
        }
    })
}
