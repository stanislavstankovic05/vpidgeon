use crate::cripto::{read_frame, write_frame};
use crate::data::Peer;
use std::net::Shutdown;
use std::sync::mpsc::Sender;
use std::sync::Arc;

pub struct Incoming {
    pub name: String,
    pub content: String,
}

/*
 * what travels inside the encrypted frame: the name, a newline, the text
 */
fn pack(name: &str, content: &str) -> Vec<u8> {
    let text = String::from(name) + "\n" + content;
    text.into_bytes()
}

fn unpack(bytes: Vec<u8>) -> Option<(String, String)> {
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
 * the session stays locked from the encryption until the frame is written,
 * so the counters and the frames keep the same order even with 2 threads sending
 */
pub fn send_to(peer: &Peer, name: &str, content: &str) -> std::result::Result<(), String> {
    let mut stream = match peer.stream.try_clone() {
        Ok(stream) => stream,
        Err(err) => {
            return Err(String::from("Couldnt use the connection: ") + err.to_string().as_str());
        }
    };
    let mut session = peer.session.lock().unwrap();
    let encrypted = session.encrypt(pack(name, content).as_slice())?;
    write_frame(&mut stream, &encrypted)
}

/*
 * except is the index of a peer that gets nothing, the one the message came from
 */
pub fn send_to_all(
    peers: &[Peer],
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
pub fn start_readers(peers: &Arc<Vec<Peer>>, is_lead: bool, sender: Sender<Incoming>) {
    let mut index = 0;
    while index < peers.len() {
        let all_peers = Arc::clone(peers);
        let reader_sender = sender.clone();
        std::thread::spawn(move || read_loop(all_peers, index, is_lead, reader_sender));
        index = index + 1;
    }
}

fn read_loop(peers: Arc<Vec<Peer>>, index: usize, is_lead: bool, sender: Sender<Incoming>) {
    let mut stream = match peers[index].stream.try_clone() {
        Ok(stream) => stream,
        Err(_) => return,
    };

    loop {
        let frame = match read_frame(&mut stream) {
            Ok(frame) => frame,
            Err(_) => break,
        };

        let decrypted = {
            let mut session = peers[index].session.lock().unwrap();
            session.decrypt(&frame)
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

pub fn close_all(peers: &[Peer]) {
    for peer in peers {
        let _ = peer.stream.shutdown(Shutdown::Both);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cripto::{handshake_lead, handshake_member};
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc::channel;
    use std::sync::Mutex;
    use std::time::Duration;

    fn make_pair() -> (Peer, Peer) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let mut member_stream = TcpStream::connect(address).unwrap();
        let (mut lead_stream, _) = listener.accept().unwrap();

        let lead_thread = std::thread::spawn(move || {
            let session = handshake_lead(&mut lead_stream, "secret").unwrap();
            Peer {
                stream: lead_stream,
                session: Arc::new(Mutex::new(session)),
            }
        });
        let session = handshake_member(&mut member_stream, "secret").unwrap();
        let member = Peer {
            stream: member_stream,
            session: Arc::new(Mutex::new(session)),
        };
        (lead_thread.join().unwrap(), member)
    }

    #[test]
    fn pack_and_unpack() {
        let bytes = pack("ana", "hello\nsecond line");
        let (name, content) = unpack(bytes).unwrap();
        assert_eq!(name, "ana");
        assert_eq!(content, "hello\nsecond line");
        assert!(unpack(b"no newline".to_vec()).is_none());
    }

    #[test]
    fn lead_relays_between_members() {
        let (lead_side_1, member_1) = make_pair();
        let (lead_side_2, member_2) = make_pair();

        let lead_peers = Arc::new(vec![lead_side_1, lead_side_2]);
        let member_1_peers = Arc::new(vec![member_1]);
        let member_2_peers = Arc::new(vec![member_2]);

        let (lead_sender, lead_receiver) = channel();
        let (member_1_sender, member_1_receiver) = channel();
        let (member_2_sender, member_2_receiver) = channel();
        start_readers(&lead_peers, true, lead_sender);
        start_readers(&member_1_peers, false, member_1_sender);
        start_readers(&member_2_peers, false, member_2_sender);

        let errors = send_to_all(&member_1_peers, None, "ana", "hi all");
        assert!(errors.is_empty());

        let wait = Duration::from_secs(5);
        let on_lead = lead_receiver.recv_timeout(wait).unwrap();
        assert_eq!(on_lead.name, "ana");
        assert_eq!(on_lead.content, "hi all");
        let on_member_2 = member_2_receiver.recv_timeout(wait).unwrap();
        assert_eq!(on_member_2.name, "ana");
        assert_eq!(on_member_2.content, "hi all");

        send_to_all(&lead_peers, None, "lead", "welcome");
        assert_eq!(member_1_receiver.recv_timeout(wait).unwrap().content, "welcome");
        assert_eq!(member_2_receiver.recv_timeout(wait).unwrap().content, "welcome");

        let echo = member_1_receiver.recv_timeout(Duration::from_millis(300));
        assert!(echo.is_err());

        close_all(&lead_peers);
        let notice = member_1_receiver.recv_timeout(wait).unwrap();
        assert_eq!(notice.name, "system");
    }

    #[test]
    fn many_messages_keep_their_order() {
        let (lead_side, member) = make_pair();
        let lead_peers = Arc::new(vec![lead_side]);
        let member_peers = Arc::new(vec![member]);
        let (lead_sender, lead_receiver) = channel();
        let (member_sender, _member_receiver) = channel();
        start_readers(&lead_peers, true, lead_sender);
        start_readers(&member_peers, false, member_sender);

        let sender_peers = Arc::clone(&member_peers);
        let first = std::thread::spawn(move || {
            for number in 0..50 {
                send_to_all(&sender_peers, None, "a", format!("a{}", number).as_str());
            }
        });
        let sender_peers = Arc::clone(&member_peers);
        let second = std::thread::spawn(move || {
            for number in 0..50 {
                send_to_all(&sender_peers, None, "b", format!("b{}", number).as_str());
            }
        });
        first.join().unwrap();
        second.join().unwrap();

        let mut got_a = 0;
        let mut got_b = 0;
        for _ in 0..100 {
            let message = lead_receiver.recv_timeout(Duration::from_secs(5)).unwrap();
            if message.name == "a" {
                assert_eq!(message.content, format!("a{}", got_a));
                got_a = got_a + 1;
            } else {
                assert_eq!(message.content, format!("b{}", got_b));
                got_b = got_b + 1;
            }
        }
        assert_eq!(got_a, 50);
        assert_eq!(got_b, 50);
    }
}
