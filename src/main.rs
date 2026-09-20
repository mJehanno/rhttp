use std::{
    error::Error,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

const ALLOWED_METHOD: &'static [&str] = &["GET", "POST", "PUT", "PATCH", "OPERATION", "HEAD"];

enum MalFormedHeaderDetail {
    InvalidHeaderLineLen,
    InvalidMethod,
    InvalidHTTPVersion,
}

enum HttpError {
    MalFormedHeader(MalFormedHeaderDetail),
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server listening on :7878");

    for incoming in listener.incoming() {
        let Ok(mut stream) = incoming else {
            panic!("Failed to connect");
        };
        let _ = handle_connection(&stream);
        let response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
    }

    Ok(())
}

fn handle_connection(stream: &TcpStream) -> Result<(), HttpError> {
    let buf_reader = BufReader::new(stream);
    let request: Vec<_> = buf_reader
        .lines()
        .map(|x| x.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("body: \n{:?}", request);
    let header_line: Vec<&str> = request[0].split(" ").collect();
    if header_line.len() != 3 {
        return Err(HttpError::MalFormedHeader(
            MalFormedHeaderDetail::InvalidHeaderLineLen,
        ));
    }

    if !ALLOWED_METHOD.contains(&header_line[0]) {
        return Err(HttpError::MalFormedHeader(
            MalFormedHeaderDetail::InvalidMethod,
        ));
    }

    if header_line[2] != "HTTP/1.1" {
        return Err(HttpError::MalFormedHeader(
            MalFormedHeaderDetail::InvalidHTTPVersion,
        ));
    }

    Ok(())
}
