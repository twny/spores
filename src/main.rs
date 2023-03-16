use std::fs;
use std::process;
use std::collections::HashMap;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

type Handler = fn(&str) -> String;

fn get_foo(_: &str) -> String {
    let status = "HTTP/1.1 404 Not Found \r\n";
    let body = "<html><h1>Foo</h1></html>";
    let size = format!("Content-Length: {}\r\n", body.len());
    let response = format!("{status}{size}\r\n{body}");
    return response;
}

fn not_found(_: &str) -> String {
    let status = "HTTP/1.1 404 Not Found \r\n";
    let body = fs::read_to_string("src/404.html").unwrap_or("".to_string());
    let size = format!("Content-Length: {}\r\n", body.len());
    let response = format!("{status}{size}\r\n{body}");
    return response;
}

fn get_index(_: &str) -> String {
    let body = fs::read_to_string("src/index.html").unwrap_or("".to_string());
    let status = "HTTP/1.1 200 OK \r\n";
    let size = format!("Content-Length: {}\r\n", body.len());
    let response = format!("{status}{size}\r\n{body}");
    return response;
}


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
    let req: Vec<String> = buf
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();


    // TODO how to memoize this hashmap for the whole run time?
    let mut routes: HashMap<String, Handler> = HashMap::new();
    routes.insert("GET / HTTP/1.1".to_string(), get_index);
    routes.insert("GET /foo HTTP/1.1".to_string(), get_foo);

    let empty_body = String::new();
    let path = req.get(0).unwrap_or(&empty_body);

    let response = match routes.get(path) {
        Some(handler) => handler(path),
        None => not_found(path),
    };

    match stream.write_all(response.as_bytes()) {
        Ok(r) => r,
        Err(err) => {
            println!("{err}");
            return;
        }
    };
}
