use std::io::{Read, Result};

use colored::Colorize;

use crate::async_net::client::TcpClient;

pub struct Response {
    client: TcpClient,
}

pub fn status_code(code: i32) -> i32 {
    match code {
        200 | 400 | 404 => code,
        _ => 501,
    }
}

pub fn status(code: i32) -> &'static str {
    match code {
        200 => "OK",
        400 => "BAD REQUEST",
        404 => "NOT FOUND",
        _ => "NOT IMPLEMENTED",
    }
}

impl Response {
    pub fn new(client: TcpClient) -> Self {
        Response { client }
    }

    pub fn parse_mime_type(&self, key: &str) -> &str {
        if let Some((_, ext)) = key.rsplit_once(".") {
            match ext {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "text/javascript",
                "jpg" => "image/jpeg",
                "jpeg" => "image/jpeg",
                "png" => "image/png",
                "ico" => "image/x-icon",
                "pdf" => "application/pdf",
                _ => "text/plain",
            }
        } else {
            "text/plain"
        }
    }

    pub fn read_response_file(&self, path: &str) -> Vec<u8> {
        let file = std::fs::File::open(String::from(path)).unwrap();
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(file);
        reader.read_to_end(&mut buf).unwrap();
        buf
    }

    pub async fn send_file(&mut self, code: i32, path: &str) -> Result<()> {
        let contents = self.read_response_file(path.clone());
        let len = contents.len();

        let mime_type = self.parse_mime_type(path);
        let content = format!(
            "HTTP/1.0 {} {}
content-type: {}; charset=UTF-8
content-length: {}

",
            status_code(code),
            status(code),
            mime_type,
            len
        );

        let bytes = content.as_bytes();
        self.client.write(&bytes).await?;
        self.client.write(&contents).await?;
        self.client.flush();

        println!(
            "{} {:?} writing response \n{}",
            format!("[{}]", std::process::id()).truecolor(0, 255, 136),
            std::thread::current().id(),
            content
        );

        Ok(())
    }
}
