use crate::cripto::{receive_payload, send_payload};
use crate::data::Connection;
use std::net::Shutdown;
use std::sync::Arc;
use std::sync::mpsc::Sender;

pub const MAX_MESSAGE: usize = 2000;

pub struct Incoming {
    pub name: String,
    pub content: String,
}

/*
 * what travels inside the encrypted message: the name, a newline, the text
 */
pub fn pack(name: &str, content: &str) -> Vec<u8> {
    let text = String::from(name) + "\n" + content;
    text.into_bytes()
}

pub fn unpack(bytes: Vec<u8>) -> Option<(String, String)> {
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => return None,
    };
    match text.split_once('\n') {
        Some((name, content)) => Some((String::from(name), String::from(content))),
        None => None,
    }
}

/*
 * the session stays locked from the encryption until the message is written,
 * so the counters and the messages keep the same order even with 2 threads sending
 */
pub fn send_to(peer: &Connection, name: &str, content: &str) -> std::result::Result<(), String> {
    if content.chars().count() > MAX_MESSAGE {
        return Err(String::from("The message is too long, the limit is ")
            + MAX_MESSAGE.to_string().as_str()
            + " characters");
    }
    let mut session = peer.session.lock().unwrap();
    let encrypted = session.encrypt(pack(name, content).as_slice())?;
    send_payload(&peer.stream, &encrypted)
}

/*
 * except is the index of a peer that gets nothing, the one the message came from
 */
pub fn send_to_all(
    peers: &[Connection],
    except: Option<usize>,
    name: &str,
    content: &str,
) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let mut index = 0;
    for peer in peers {
        let skip = match except {
            Some(value) => value == index,
            None => false,
        };
        if !skip {
            match send_to(peer, name, content) {
                Ok(_) => {}
                Err(message) => errors.push(message),
            }
        }
        index = index + 1;
    }
    errors
}

/*
 * one thread for each peer, waits for a message and sends it to the screen.
 * the lead also sends it to the other members.
 */
pub fn start_readers(peers: &Arc<Vec<Connection>>, is_lead: bool, sender: Sender<Incoming>) {
    let mut index = 0;
    while index < peers.len() {
        let all_peers = Arc::clone(peers);
        let reader_sender = sender.clone();
        std::thread::spawn(move || read_loop(all_peers, index, is_lead, reader_sender));
        index = index + 1;
    }
}

fn read_loop(peers: Arc<Vec<Connection>>, index: usize, is_lead: bool, sender: Sender<Incoming>) {
    loop {
        let message = match receive_payload(&peers[index].stream) {
            Ok(message) => message,
            Err(_) => break,
        };

        let decrypted = {
            let mut session = peers[index].session.lock().unwrap();
            session.decrypt(&message)
        };
        let plain = match decrypted {
            Ok(plain) => plain,
            Err(_) => break,
        };

        match unpack(plain) {
            Some((name, content)) => {
                if is_lead {
                    send_to_all(&peers, Some(index), name.as_str(), content.as_str());
                }
                let _ = sender.send(Incoming { name, content });
            }
            None => {}
        }
    }

    let notice = if is_lead {
        "A member left the chat"
    } else {
        "The lead left, the chat is over"
    };
    let _ = sender.send(Incoming {
        name: String::from("system"),
        content: String::from(notice),
    });
}

pub fn close_all(peers: &[Connection]) {
    for peer in peers {
        let _ = peer.stream.shutdown(Shutdown::Both);
    }
}
