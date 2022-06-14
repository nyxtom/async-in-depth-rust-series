mod node;
mod response;
mod router;
mod routes;
mod thread_pool;
mod worker;

use crate::thread_pool::ThreadPool;
use colored::*;
use router::Router;
use std::io::{Error, Result};
use std::net::TcpListener;
use std::sync::Arc;

fn check_err<T: Ord + Default>(num: T) -> Result<T> {
    if num < T::default() {
        return Err(Error::last_os_error());
    }
    Ok(num)
}

fn fork() -> Result<u32> {
    check_err(unsafe { libc::fork() }).map(|pid| pid as u32)
}

fn wait(pid: i32) -> Result<u32> {
    check_err(unsafe { libc::waitpid(pid, 0 as *mut libc::c_int, 0) }).map(|code| code as u32)
}

fn main() -> Result<()> {
    let port = std::env::var("PORT").unwrap_or(String::from("7000"));
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    println!(
        "{} server listening on 127.0.0.1:{}",
        format!("[{}]", std::process::id()).truecolor(0, 255, 136),
        port
    );

    let mut router = Router::new();
    routes::configure(&mut router);
    let router = Arc::new(router);
    let mut pids = vec![];
    for _ in 0..2 {
        let child_pid = fork()?;
        if child_pid == 0 {
            let pool = ThreadPool::new(4);
            for client in listener.incoming() {
                if let Ok(client) = client {
                    let router = Arc::clone(&router);
                    pool.execute(move || {
                        println!(
                            "{} [{:?}] client connected at {}",
                            format!("[{}]", std::process::id()).truecolor(0, 255, 136),
                            std::thread::current().id(),
                            client.peer_addr().unwrap()
                        );
                        router.route_client(client).unwrap();
                    });
                }
            }
            break;
        } else {
            println!(
                "{} forking process, new {child_pid}",
                format!("[{}]", std::process::id()).truecolor(0, 255, 136)
            );
        }
        pids.push(child_pid);
    }

    for p in pids {
        wait(p as i32)?;
        println!(
            "{} <<< {p} exit()",
            format!("[{}]", std::process::id()).truecolor(200, 255, 136)
        );
    }

    Ok(())
}
