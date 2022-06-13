use std::{
    fmt::Display,
    io::{BufWriter, Result, Write},
    net::TcpStream,
};

pub struct Response {
    writer: BufWriter<TcpStream>,
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
    pub fn new(client: TcpStream) -> Self {
        Response {
            writer: BufWriter::new(client),
        }
    }

    pub fn into_inner(self) -> Result<TcpStream> {
        match self.writer.into_inner() {
            Ok(s) => Ok(s),
            Err(e) => Err(e.into_error()),
        }
    }

    pub fn write_status(&mut self, code: i32) -> Result<usize> {
        let code = status_code(code);
        self.writer
            .write(format!("HTTP/1.0 {} {}\n", code, status(code)).as_bytes())
    }

    pub fn write_header<V: Display>(&mut self, key: &str, val: V) -> Result<usize> {
        self.writer
            .write(format!("\"{}\": {}\n", key, val).as_bytes())
    }

    pub fn write_body(&mut self, val: &[u8]) -> Result<usize> {
        self.write_header("content-length", val.len())?;
        self.writer.write(b"\n")?;
        self.writer.write(val)
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

    pub fn send_file_contents(&mut self, code: i32, path: &str, contents: Vec<u8>) -> Result<()> {
        self.write_status(code)?;
        let len = contents.len();
        let mime_type = self.parse_mime_type(path);
        self.write_header("content-type", format!("{}; charset=UTF-8", mime_type))?;
        self.write_header("content-length", len)?;
        self.write_body(&contents)?;
        self.flush()
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}
