use std::net::TcpStream;
use std::string::*;

struct User {
    private_ip: String,
    public_ip: String,
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

/*
 * both functions return (send_key, receive_key) for this stream
 */
pub fn handshake_member(
    stream: &mut TcpStream,
    psk: &str,
) -> std::result::Result<([u8; 32], [u8; 32]), String> {
    todo!()
}

pub fn handshake_lead(
    stream: &mut TcpStream,
    psk: &str,
) -> std::result::Result<([u8; 32], [u8; 32]), String> {
    todo!()
}
