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
