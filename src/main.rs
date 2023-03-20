use std::collections::HashMap;
use std::process;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

// TODO: should pass Method or ParsedRequest instead of &str, and change the name of the function so it's not get_, since it's not just for GET requests
type Handler = fn(&str) -> String;

const BODY_404: &str = include_str!("404.html");
const BODY_INDEX: &str = include_str!("index.html");

fn not_found(_: &str) -> String {
    let body = BODY_404;
    response(body, "404 Not Found")
}

fn get_sleep(_: &str) -> String {
    let body = "<html><h1>Sleeeeepy</h1></html>";
    thread::sleep(Duration::from_secs(10));
    response(body, "200 Ok")
}

fn get_index(_: &str) -> String {
    let body = BODY_INDEX;
    response(body, "200 Ok")
}

fn response(body: &str, status: &str) -> String {
    let status = format!("HTTP/1.1 {status} \r\n");
    let size = format!("Content-Length: {}\r\n", body.len());
    format!("{status}{size}\r\n{body}")
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    let mut routes: HashMap<&str, Handler> = HashMap::new();
    routes.insert("/", get_index);
    routes.insert("/sleep", get_sleep);

    let routes = Arc::new(routes);

    for stream in listener.incoming() {
        /* Arc clone let's us create a new reference to the same data
        instead of cloning the data itself */
        let routes = Arc::clone(&routes);
        let stream = stream.unwrap();

        // TODO make this a thead pool
        thread::spawn(move || {
            handle_connection(stream, routes);
        });

        println!("Connection established!");
    }
}

fn handle_connection(mut stream: TcpStream, routes: Arc<HashMap<&str, Handler>>) {
    let mut reader = BufReader::new(&stream);
    let mut req: Vec<String> = reader
        .by_ref()
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    match req[0].contains("POST") {
        true => {
            println!("POST request");
            let mut contents_raw: Vec<u8> = vec![];
            reader.read_until(b'}', &mut contents_raw).unwrap();
            req.push(String::from_utf8(contents_raw).unwrap());
        }
        false => {
            println!("GET request");
        }
    };

    let parsed_request = parse_request(&req);
    let path = parsed_request.route.as_str();

    let response = match routes.get(path) {
        Some(handler) => handler(path),
        None => not_found(path),
    };

    match stream.write_all(response.as_bytes()) {
        Ok(r) => r,
        Err(err) => {
            println!("{err}");
        }
    };
}

/// Parses the request string into a ParsedRequest struct
///
/// Gets the route from the request string, e.g. "/foo/bar?baz=qux" -> "/foo/bar"
///
/// # Examples
///
/// ```
/// let request = "GET /foo/bar?baz=qux HTTP/1.1";
/// let route = get_parsed_request(request);
/// assert_eq!(route, "/foo/bar");
/// ```
fn parse_request(request: &[String]) -> ParsedRequest {
    let request_line = match request.first() {
        Some(r) => r,
        None => "",
    };

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap();
    let route = parts.next().unwrap();
    let version = parts.next().unwrap();
    let mut route_parts = route.split('?');
    let route = route_parts.next().unwrap();
    let query = route_parts.next().unwrap_or("");
    let mut headers = HashMap::new();

    for (index, header) in request.iter().enumerate() {
        if index > 0 {
            if !header.contains(':') || header.starts_with('{') {
                continue;
            }
            let mut split = header.split(':');
            headers.insert(
                // key
                split.next().unwrap().trim().to_string(),
                // value
                split.collect::<Vec<&str>>().join(":").trim().to_string(),
            );
        }
    }

    let method = match method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => Method::GET,
    };

    let body = match method {
        Method::POST => match request.last() {
            Some(r) => {
                if r.starts_with('{') {
                    r
                } else {
                    ""
                }
            }
            None => "",
        },
        _ => "",
    };

    ParsedRequest {
        method,
        route: route.to_string(),
        version: version.to_string(),
        query: query.to_string(),
        headers,
        body: body.to_string(),
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq)]
enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug, PartialEq)]
struct ParsedRequest {
    method: Method,
    route: String,
    version: String,
    query: String,
    headers: HashMap<String, String>,
    body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_parsed_request() {
        let request = vec!["GET / HTTP/1.1".to_string()];
        let parsed = parse_request(&request);
        assert_eq!(parsed.route, "/");

        let request = vec!["GET /foo HTTP/1.1".to_string()];
        let parsed = parse_request(&request);
        assert_eq!(parsed.route, "/foo");

        let request = vec!["GET /foo/bar HTTP/1.1".to_string()];
        let parsed = parse_request(&request);
        assert_eq!(parsed.route, "/foo/bar");

        let request = vec!["GET /foo/bar?baz=qux HTTP/1.1".to_string()];
        let parsed = parse_request(&request);
        assert_eq!(parsed.route, "/foo/bar");

        let request = vec![
            "GET /foo/bar?baz=qux HTTP/1.1".to_string(),
            "Host: localhost:7878".to_string(),
        ];
        let parsed = parse_request(&request);
        assert_eq!(parsed.headers.get("Host").unwrap(), "localhost:7878");
    }
}
