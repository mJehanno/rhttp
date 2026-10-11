pub mod session {
    use std::{
        fs,
        io::{BufReader, prelude::*},
        net::TcpListener,
    };

    use crate::router::{self, Router, Routing, RoutingGroup};
    const ALLOWED_METHOD: &'static [&str] = &["GET", "POST", "PUT", "PATCH", "OPTION", "HEAD"];

    pub struct Server {
        pub router: Router,
        private: bool,
    }

    impl Server {
        pub fn new() -> Server {
            Server {
                router: router::Router::new(),
                private: true,
            }
        }

        pub fn start(&self) {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            println!("Server listening on :7878");
            for incoming in listener.incoming() {
                let Ok(mut stream) = incoming else {
                    panic!("Failed to connect");
                };

                // parse the url + method, give it to router
                let buf_reader = BufReader::new(&stream);
                let request: Vec<_> = buf_reader
                    .lines()
                    .map(|x| x.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();

                println!("body: \n{:?}", request);

                let header_line: Vec<&str> = request[0].split(" ").collect();

                // malformed header line
                if header_line.len() != 3 {}

                // invalid HTTP Method
                if !ALLOWED_METHOD.contains(&header_line[0]) {}

                let (method, path) = (header_line[0], header_line[1]);

                let fragments: Vec<String> =
                    path.split("/").map(|x| String::from("/") + x).collect();

                let status_line = "HTTP/1.1 200 OK";
                for r in self.router.groups() {
                    match check_router(0, r, &fragments) {
                        Some(route) if route.method == method.to_owned().into() => {
                            let index = fs::read_to_string("index.html").unwrap();
                            let response = format!(
                                "{status_line}\r\nContent-Length: {}\r\n\r\n{index}",
                                index.len()
                            );
                            stream.write_all(response.as_bytes()).unwrap();
                        }
                        Some(_) => {
                            let status_line = "HTTP/1.1 405 METHOD NOT ALLOWED";
                            let response = format!("{status_line}\r\n\r\n");
                            stream.write_all(response.as_bytes()).unwrap();
                        }
                        None => {
                            let status_line = "HTTP/1.1 404 NOT FOUND";
                            let response = format!("{status_line}\r\n\r\n");
                            stream.write_all(response.as_bytes()).unwrap();
                        }
                    }
                }
            }
        }
    }

    // should return an Option
    fn check_router<'a>(
        depth: i32,
        rg: &'a RoutingGroup,
        fragments: &Vec<String>,
    ) -> Option<&'a router::Route> {
        if rg.prefix().to_owned() != fragments[depth as usize] {
            return None;
        }

        let Some(route) = rg
            .routes()
            .iter()
            .find(|x| x.path == fragments[depth as usize])
        else {
            return check_router(depth + 1, rg, fragments);
        };
        Some(route)
    }
}

pub mod router {
    use std::{error::Error, net::TcpListener};

    #[derive(PartialEq)]
    pub enum HTTPMethod {
        Get,
        Post,
        Put,
        Patch,
        Delete,
    }

    // best practice should impl try from
    impl From<String> for HTTPMethod {
        fn from(value: String) -> Self {
            match value.as_str() {
                "GET" => HTTPMethod::Get,
                "POST" => HTTPMethod::Post,
                "PUT" => HTTPMethod::Put,
                "PATCH" => HTTPMethod::Patch,
                "DELETE" => HTTPMethod::Delete,
                _ => panic!("unknown http method"),
            }
        }
    }

    pub type HTTPHandler = fn(req: HTTPRequest) -> HTTPResponse;

    pub trait Routing {
        fn groups(&self) -> &Vec<Box<RoutingGroup>>;
        fn group(&mut self, prefix: String);
        fn get(&mut self, path: String, handler: HTTPHandler);
        fn post(&mut self, path: String, handler: HTTPHandler);
        fn delete(&mut self, path: String, handler: HTTPHandler);
        fn patch(&mut self, path: String, handler: HTTPHandler);
        fn put(&mut self, path: String, handler: HTTPHandler);
    }

    pub struct Router {
        groups: Vec<Box<RoutingGroup>>,
    }

    impl Router {
        //
        pub fn new() -> Self {
            Router {
                groups: Vec::from([Box::new(RoutingGroup {
                    prefix: String::from("/"),
                    routes: Vec::new(),
                    groups: Vec::new(),
                })]),
            }
        }

        //
        pub fn start(port: u64) -> Result<(), Box<dyn Error>> {
            let _ = TcpListener::bind(format!("127.0.0.1:{port}"))?;
            println!("Server listening on :{port}");
            Ok(())
        }
    }

    impl Routing for Router {
        fn groups(&self) -> &Vec<Box<RoutingGroup>> {
            &self.groups
        }
        //
        //
        fn group(&mut self, prefix: String) {
            let group = Box::new(RoutingGroup {
                prefix,
                routes: Vec::new(),
                groups: Vec::new(),
            });
            self.groups.push(group);
        }

        //
        fn get(&mut self, path: String, handler: HTTPHandler) {
            self.groups[0].get(path, handler);
        }

        //
        fn put(&mut self, path: String, handler: HTTPHandler) {
            self.groups[0].put(path, handler);
        }

        //
        fn post(&mut self, path: String, handler: HTTPHandler) {
            self.groups[0].post(path, handler);
        }

        //
        fn patch(&mut self, path: String, handler: HTTPHandler) {
            self.groups[0].patch(path, handler);
        }

        //
        fn delete(&mut self, path: String, handler: HTTPHandler) {
            self.groups[0].delete(path, handler);
        }
    }

    impl Routing for RoutingGroup {
        fn groups(&self) -> &Vec<Box<RoutingGroup>> {
            &self.groups
        }

        //
        fn group(&mut self, prefix: String) {
            let group = Box::new(RoutingGroup {
                prefix,
                routes: Vec::new(),
                groups: Vec::new(),
            });
            self.groups.push(group);
        }

        //
        fn get(&mut self, path: String, handler: HTTPHandler) {
            self.routes.push(Route {
                method: HTTPMethod::Get,
                path,
                handler,
            });
        }

        //
        fn put(&mut self, path: String, handler: HTTPHandler) {
            self.routes.push(Route {
                method: HTTPMethod::Put,
                path,
                handler,
            });
        }

        //
        fn post(&mut self, path: String, handler: HTTPHandler) {
            self.routes.push(Route {
                method: HTTPMethod::Post,
                path,
                handler,
            });
        }

        //
        fn patch(&mut self, path: String, handler: HTTPHandler) {
            self.routes.push(Route {
                method: HTTPMethod::Patch,
                path,
                handler,
            });
        }

        //
        fn delete(&mut self, path: String, handler: HTTPHandler) {
            self.routes.push(Route {
                method: HTTPMethod::Delete,
                path,
                handler,
            });
        }
    }

    pub struct RoutingGroup {
        prefix: String,
        groups: Vec<Box<RoutingGroup>>,
        routes: Vec<Route>,
    }

    impl RoutingGroup {
        pub fn prefix(&self) -> &str {
            &self.prefix
        }

        pub fn routes(&self) -> &Vec<Route> {
            &self.routes
        }
    }

    // - header
    // - body
    pub struct HTTPRequest {}

    // - header
    // - status
    // - body
    pub struct HTTPResponse {}

    pub struct Route {
        pub method: HTTPMethod,
        pub path: String,
        pub handler: HTTPHandler,
    }

    // post,put,patch, group
    // let v1 = router.group("v1")
    // v1.post()
}
pub mod parser {}
