use std::process;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

type Handler = fn(&str) -> String;

const BODY_404: &str = include_str!("404.html");
const BODY_INDEX: &str = include_str!("index.html");

fn not_found(_: &str) -> String {
    let body = BODY_404.to_string();
    return response(&body, &"404 Not Found".to_string());
}

fn get_sleep(_: &str) -> String {
    let body = "<html><h1>Sleeeeepy</h1></html>";
    thread::sleep(Duration::from_secs(10));
    return response(&body, &"200 Ok".to_string());
}

fn get_index(_: &str) -> String {
    let body = BODY_INDEX.to_string();
    return response(&body, &"200 Ok".to_string());
}

fn response(body: &str, status: &str) -> String {
    let status = format!("HTTP/1.1 {status} \r\n");
    let size = format!("Content-Length: {}\r\n", body.len());
    return format!("{status}{size}\r\n{body}");
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // TODO make this a thead pool
        thread::spawn(|| {
            handle_connection(stream);
        });

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
    routes.insert("GET /sleep HTTP/1.1".to_string(), get_sleep);

    let path = req.get(0).cloned().unwrap_or_default();

    let response = match routes.get(&path) {
        Some(handler) => handler(&path),
        None => not_found(&path),
    };

    match stream.write_all(response.as_bytes()) {
        Ok(r) => r,
        Err(err) => {
            println!("{err}");
            return;
        }
    };
}
