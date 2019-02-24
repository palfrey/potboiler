use actix_web::{http::Method, test::TestServer, App, HttpRequest, HttpResponse};
use std::{
    fmt,
    ops::DerefMut,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct RecordRequest {
    pub path: String,
    pub body: String,
    pub method: Method,
}

pub struct RecordServer {
    pub requests: Arc<RwLock<Vec<RecordRequest>>>,
    pub server: TestServer,
}

impl fmt::Debug for RecordServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RecordServer: requests {:?}", self.requests)
    }
}

impl RecordServer {
    fn recording_server(requests: Arc<RwLock<Vec<RecordRequest>>>) -> App {
        App::new().default_resource(move |r| {
            r.route().with(move |(req, body): (HttpRequest, String)| {
                requests.clone().write().unwrap().deref_mut().push(RecordRequest {
                    path: req.path().to_string(),
                    body: body,
                    method: req.method().clone(),
                });
                HttpResponse::MethodNotAllowed()
            })
        })
    }

    pub fn new() -> RecordServer {
        let requests = Arc::new(RwLock::new(Vec::new()));
        RecordServer {
            requests: requests.clone(),
            server: TestServer::with_factory(move || RecordServer::recording_server(requests.clone())),
        }
    }
}
