use std::net::IpAddr::V4;
use std::net::Ipv4Addr;
use std::net::*;
use std::string::*;

pub struct User {
    pub private_ip: Ipv4Addr,
    pub public_ip: Ipv4Addr,
}

impl User {
    fn get_private_ip() {}

    pub fn instance_user(self: &mut User) {
        /*
         * CONNECT TO LOCAL HOST TO GET THE IP
         */
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");

        let ip = stream.local_addr().unwrap().ip();
        println!("ip: {}", ip);
        match ip {
            IpAddr::V4(ip) => self.public_ip = ip,
            _ => println!("invalid ip (not IPv4"),
        }
        // self.public_ip = ip;

        // self.public_ip = stream.local_addr().unwrap();
    }
}
struct Message {
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
