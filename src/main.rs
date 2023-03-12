use std::fs;
use std::process;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);

        println!("Connection established!");
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf = BufReader::new(&mut stream);
    let req: Vec<_> = buf
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request {:#?}", req);
    let body = match fs::read_to_string("src/root.html") {
        Ok(r) => r,
        Err(err) => {
            println!("{err}");
            return;
        }
    };
    let status = "HTTP/1.1 200 OK \r\n";
    let size = format!("Content-Length: {}\r\n", body.len());
    let response = format!("{status}{size}\r\n{body}");
    match stream.write_all(response.as_bytes()) {
        Ok(r) => r,
        Err(err) => {
            println!("{err}");
            return;
        }
    }
}
