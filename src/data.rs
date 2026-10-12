use std::net::IpAddr::V4;
use std::net::Ipv4Addr;
use std::net::*;
use std::string::*;
use whoami::*;
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
    sender: User,
    reply_to: Option<Box<Message>>,
    content: String,
}
/*
 * Messages will be stored locally.
 */

struct Connection {
    self_peer: User,
    peer: User,
    message_list: Vec<Message>,
    crypto: String,
    key: String,
}

// impl Connection {
//     fn get_private_ip(),
//     fn create_handshake(),
// }
