use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use hello_web::ThreadPool;

const HTTP_HEADER: &str = "HTTP/1.1";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn starts_with(buffer: &[u8], method: &str) -> bool {
    buffer.starts_with(format!("{} {}", method, HTTP_HEADER).as_bytes())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let _ = stream.read(&mut buffer).unwrap();

    let mut content_type = "".to_string();

    let (status_line, filename) = if starts_with(&buffer, "GET /") {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if starts_with(&buffer, "GET /sleep") {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else if starts_with(&buffer, "GET /favicon.ico") {
        content_type.push_str("Content-Type: image/svg+xml\r\n");
        ("HTTP/1.1 200 OK", "rust-language-icon.svg")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n{}\r\n{}",
        status_line,
        contents.len(),
        content_type,
        contents
    );

    let _ = stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
