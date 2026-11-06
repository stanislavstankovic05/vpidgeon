use std::net::IpAddr::V4;
use std::net::Ipv4Addr;
use std::net::*;
use std::string::*;
use std::sync::{Arc, Mutex};
use whoami::*;
#[derive(Clone)]
pub struct User {
    pub private_ip: Ipv4Addr,
    pub public_ip: Ipv4Addr,

    pub secret_key: String,
    pub pub_key: String,
    pub name: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            private_ip: (Ipv4Addr::LOCALHOST),
            public_ip: (Ipv4Addr::LOCALHOST),
            secret_key: (String::new()),
            pub_key: (String::new()),
            name: (whoami::username().unwrap()),
        }
    }
}
pub struct Message {
    pub sender: User,
    pub reply_to: Option<Box<Message>>,
    pub content: String,
}

pub struct Connection {
    pub self_peer: User,
    pub peer: User,
    pub message_list: Vec<Message>,
    pub stream: TcpStream,
    pub session: Arc<Mutex<Session>>,
}

// impl Connection {
//     fn get_private_ip(),
//     fn create_handshake(),
// }

impl Connection {
    pub fn new(stream: TcpStream, session: Session) -> Self {
        let mut peer = User::default();
        peer.name = String::new();
        Self {
            self_peer: (User::default()),
            peer: (peer),
            message_list: (Vec::new()),
            stream: (stream),
            session: (Arc::new(Mutex::new(session))),
        }
    }
}

pub struct Session {
    pub transport: snow::TransportState,
}

impl Session {
    pub fn encrypt(&mut self, plaintext: &[u8]) -> std::result::Result<Vec<u8>, String> {
        let mut buffer: Vec<u8> = vec![0; plaintext.len() + 16];
        match self.transport.write_message(plaintext, &mut buffer) {
            Ok(length) => {
                buffer.truncate(length);
                Ok(buffer)
            }
            Err(err) => Err(String::from("Couldnt encrypt: ") + err.to_string().as_str()),
        }
    }

    pub fn decrypt(&mut self, ciphertext: &[u8]) -> std::result::Result<Vec<u8>, String> {
        let mut buffer: Vec<u8> = vec![0; ciphertext.len()];
        match self.transport.read_message(ciphertext, &mut buffer) {
            Ok(length) => {
                buffer.truncate(length);
                Ok(buffer)
            }
            Err(err) => Err(String::from("Couldnt decrypt: ") + err.to_string().as_str()),
        }
    }
}
