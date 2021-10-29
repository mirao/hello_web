use hello_web::log;
use hello_web::ThreadPool;

use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::stdout;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // Don't block processing of input (key press) when waiting for incoming connection
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    // Web server in max 4 threads
    let pool = ThreadPool::new(4).unwrap();

    println!("Press 'q' to shutdown web server");
    let (send_key, recv_key) = mpsc::channel();

    // Read key from input and send to channel
    // Processing of input in thread doesn't block stdout, therefore workers can still log on screen
    // Raw mode set for stdout requires '\r' in println! to get correct EOLN
    let _stdout = stdout().into_raw_mode().unwrap();
    thread::spawn(move || {
        for key in io::stdin().keys() {
            send_key.send(key.unwrap()).unwrap();
        }
    });

    for stream in listener.incoming() {
        if let Ok(s) = stream {
            pool.execute(|| {
                handle_connection(s);
            });
        }

        // Receive pressed key from channel, 'q' means triggers shutdown of web server
        if let Ok(key) = recv_key.try_recv() {
            if key == Key::Char('q') {
                break;
            }
        }
    }
    log("Shutting down.".to_string());
}

fn starts_with(buffer: &[u8], method: &str) -> bool {
    buffer.starts_with(format!("{} {}", method, "HTTP/1.1").as_bytes())
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
