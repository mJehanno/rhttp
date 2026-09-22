use regex::regex;
use std::{
    error::Error,
    fmt::Display,
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

const ALLOWED_METHOD: &'static [&str] = &["GET", "POST", "PUT", "PATCH", "OPTION", "HEAD"];

enum MalFormedHeaderDetail {
    InvalidHeaderLineLen,
    InvalidMethod,
    InvalidHTTPVersion,
    InvalidURL,
}

impl Display for MalFormedHeaderDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidHeaderLineLen => {
                "invalid header line length. a valid headerline contains 3 segments splitted by spaces"
            }
            Self::InvalidMethod => &format!(
                "invalid method, a valid method is one of {}",
                ALLOWED_METHOD.join(",")
            ),
            Self::InvalidHTTPVersion => "invalid HTTP version, the supported version is 'HTTP/1.1'",
            Self::InvalidURL => "invalid URL format",
        };
        write!(f, "{}", message)
    }
}

enum HttpError {
    MalFormedHeader(MalFormedHeaderDetail),
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalFormedHeader(x) => write!(f, "{}", x),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server listening on :7878");

    let index = fs::read_to_string("index.html")?;

    for incoming in listener.incoming() {
        let Ok(mut stream) = incoming else {
            panic!("Failed to connect");
        };
        match handle_connection(&stream) {
            Ok(_) => {
                let status_line = "HTTP/1.1 200 OK";
                let response = format!(
                    "{status_line}\r\nContent-Length: {}\r\n\r\n{index}",
                    index.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(err) => {
                let status_line = "HTTP/1.1 400 BAD REQUEST";
                let response = format!(
                    "{status_line}\r\nContent-Length: {}\r\n\r\n{err}",
                    err.to_string().len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
    }

    Ok(())
}

// TODO: support query param, url routing, body mapping from json(Content-type header ?)
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

    // TODO: parse url -> start with a / then * (0+) fragment, optional ? with key-value pair
    // speratated by comma
    // step 1 validate
    // step 2 parse
    // support for dynamic param ?? (ex: /users/id/8)
    let reg = regex!(r"^/(\w+/{0,1})*\?{0,1}(\w*=\w*,)*(\w*=\w*){0,1}");

    if !reg.is_match(header_line[1]) {
        return Err(HttpError::MalFormedHeader(
            MalFormedHeaderDetail::InvalidURL,
        ));
    }

    if header_line[2] != "HTTP/1.1" {
        return Err(HttpError::MalFormedHeader(
            MalFormedHeaderDetail::InvalidHTTPVersion,
        ));
    }

    Ok(())
}
