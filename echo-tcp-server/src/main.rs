use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() -> std::io::Result<()> {
    // fd = socket
    // bind(fd, socketaddr *, sizeof(serveraddr *))
    // listen(fd)
    let listener = TcpListener::bind("127.0.0.1:7000")?;
    println!("\nserver socket bind/listen 127.0.0.1:7000\n");

    loop {
        match listener.accept() {
            Ok((mut client_socket, addr)) => {
                println!("client {addr} connected to server");
                loop {
                    let mut bytes = [0u8; 32];
                    let res = client_socket.read(&mut bytes)?;
                    if res > 0 {
                        client_socket.write(&bytes[0..res as usize])?;
                    } else {
                        break;
                    }
                }
            }
            Err(e) => println!("Could not accept client {:?}", e),
        }
    }
}
