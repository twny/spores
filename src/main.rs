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
    let req: Vec<_> = buf.lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request {:#?}", req);
    let resp = "HTTP/1.1 200 OK \r\n\r\n";
    stream.write_all(resp.as_bytes()).unwrap();

}
