use std::{
    fs,
    io::prelude::*,
    net::TcpStream,
    io::Error,
};

#[macro_export]
macro_rules! vprintln {
    ($($arg:tt)*) => {
        #[cfg(feature = "verbose")]
        println!($($arg)*);
    };
}


const HTTP_VERSION: &str = "HTTP/1.1";

pub fn write_response(stream: &mut TcpStream, status: u16, message: Option<&str>, contents: Option<String>) {
    let http_version= HTTP_VERSION;
    let contents = contents.unwrap_or("".to_string());
    let length = contents.len();
    let message = message.unwrap_or("");
    let response = format!("{http_version} {status} {message}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

const FRONTEND_DIR: &str = "frontend";

pub fn get_frontend(file: &str) -> Result<String, Error> {
    return fs::read_to_string(format!("{FRONTEND_DIR}/{file}"));
}