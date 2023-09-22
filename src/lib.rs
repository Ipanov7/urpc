use serde::{de, Deserialize, Serialize};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    thread, usize,
    collections::HashMap,
};

pub trait UrpcService {
    fn dispatch(&mut self, method_name: String, args: String);
}

struct DefaultUrpcService {}

impl UrpcService for DefaultUrpcService {
    fn dispatch(&mut self, method_name: String, _args: String) {
        panic!("method {} not implemented", method_name)
    }
}

type UrpcServiceSync = Arc<Mutex<dyn UrpcService + Send + 'static>>;

pub struct UrpcServer {
    socket: UdpSocket,
    services: HashMap<String, UrpcServiceSync>,
}

#[derive(Serialize, Deserialize)]
pub struct UrpcRequest {
    pub service: String,
    pub method: String,
    pub message: String,
}

impl UrpcServer {
    pub fn new(address: &str) -> UrpcServer {
        let socket = UdpSocket::bind(address).expect("couldn't bind to address");
        UrpcServer { socket, services: HashMap::new() }
    }

    pub fn register(&mut self, service_name: String, service: impl UrpcService + Send + 'static) {
        if self.services.contains_key(&service_name) {
            panic!("service {} already defined!", service_name)
        }
        self.services.insert(service_name, Arc::new(Mutex::new(service)));
    }

    pub fn start(server: UrpcServer) {
        thread::spawn(move || loop {
            let mut buf = [0; 1024];
            let (number_of_bytes, src_addr) = server 
                .socket
                .recv_from(&mut buf)
                .expect("didn't receive data");

            let _ = server.handle_connection(buf, number_of_bytes, src_addr);
        });
    }

    fn handle_connection(
        &self,
        buf: [u8; 1024],
        size: usize,
        _: SocketAddr,
    ) -> Result<(), &'static str> {
        let request: UrpcRequest =
            serde_json::from_slice(&buf[..size]).unwrap_or_else(|err| panic!("{err}"));

        let service_name = request.service;
        let method_name = request.method;
        let args = request.message.clone();

        println!("Received request: {} with message: {}", method_name, args);

        if !self.services.contains_key(&service_name) {
           panic!("missing service definition for {}", service_name); 
        }

        self.services.get(&service_name).unwrap().lock().unwrap().dispatch(method_name, args);

        Ok(())
    }
}

pub fn decode<D: de::DeserializeOwned>(message: String) -> D {
    serde_json::from_str(message.as_str()).unwrap_or_else(|err| panic!("{err}"))
}

pub struct UrpcClient {
    socket: UdpSocket,
    service: String,
    recipients: Vec<SocketAddr>,
}

impl UrpcClient {
    pub fn new(service: String, recipients_addr: Vec<&String>) -> UrpcClient {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to client address");

        let mut recipients = Vec::new();
        for address in recipients_addr {
            let recipient = address.parse().expect("couldn't parse recipient address");
            recipients.push(recipient);
        }

        UrpcClient { socket, service, recipients }
    }

    pub fn send<T: Serialize>(&self, method: &str, message: T) {
        let request = UrpcRequest {
            service: self.service.clone(),
            method: method.into(),
            message: encode(message),
        };
        let encoded_req = serde_json::to_string(&request).unwrap_or_else(|err| panic!("{err}"));

        for recipient in &self.recipients {
            self.socket
                .send_to(encoded_req.as_bytes(), recipient)
                .expect("couldn't send data");
        }
    }
}

fn encode(args: impl Serialize) -> String {
    serde_json::to_string(&args).unwrap_or_else(|err| panic!("{err}"))
}
