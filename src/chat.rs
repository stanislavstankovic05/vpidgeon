use crate::cripto::{read_frame, write_frame};
use crate::data::Peer;
use std::sync::mpsc::Sender;

pub type Incoming = (usize, std::result::Result<String, String>);

pub fn pack(name: &str, content: &str) -> String {
    String::from(name) + "\n" + content
}

pub fn unpack(text: &str) -> (String, String) {
    match text.split_once('\n') {
        Some((name, content)) => (String::from(name), String::from(content)),
        None => (String::from("unknown"), String::from(text)),
    }
}

pub fn send_text(peer: &mut Peer, text: &str) -> std::result::Result<(), String> {
    let encrypted = peer.session.lock().unwrap().encrypt(text.as_bytes())?;
    write_frame(&mut peer.stream, &encrypted)
}

pub fn send_to_all(peers: &mut Vec<Peer>, except: Option<usize>, text: &str) {
    let mut index = 0;
    for peer in peers.iter_mut() {
        if Some(index) != except {
            let _ = send_text(peer, text);
        }
        index = index + 1;
    }
}

pub fn start_readers(peers: &Vec<Peer>, sender: &Sender<Incoming>) {
    let mut index = 0;
    for peer in peers.iter() {
        let mut stream = peer.stream.try_clone().unwrap();
        let session = peer.session.clone();
        let sender = sender.clone();
        let number = index;
        std::thread::spawn(move || {
            loop {
                let frame = match read_frame(&mut stream) {
                    Ok(frame) => frame,
                    Err(message) => {
                        let _ = sender.send((number, Err(message)));
                        return;
                    }
                };
                let decrypted = session.lock().unwrap().decrypt(&frame);
                match decrypted {
                    Ok(plain) => {
                        let text = String::from_utf8_lossy(&plain).to_string();
                        let _ = sender.send((number, Ok(text)));
                    }
                    Err(message) => {
                        let _ = sender.send((number, Err(message)));
                        return;
                    }
                }
            }
        });
        index = index + 1;
    }
}

pub fn close_all(peers: &mut Vec<Peer>) {
    for peer in peers.iter() {
        let _ = peer.stream.shutdown(std::net::Shutdown::Both);
    }
    peers.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cripto::{handshake_lead, handshake_member};
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc::channel;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn make_pair(listener: &TcpListener, psk: &str) -> (Peer, Peer) {
        let address = listener.local_addr().unwrap();
        let psk_copy = String::from(psk);
        let member_thread = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(address).unwrap();
            let session = handshake_member(&mut stream, psk_copy.as_str()).unwrap();
            Peer {
                address: String::from("lead"),
                stream: stream,
                session: Arc::new(Mutex::new(session)),
            }
        });
        let (mut lead_stream, _) = listener.accept().unwrap();
        let session = handshake_lead(&mut lead_stream, psk).unwrap();
        let lead_side = Peer {
            address: String::from("member"),
            stream: lead_stream,
            session: Arc::new(Mutex::new(session)),
        };
        let member_side = member_thread.join().unwrap();
        (lead_side, member_side)
    }

    #[test]
    fn member_message_reaches_the_lead_and_the_other_member() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let (lead_to_one, one) = make_pair(&listener, "secret");
        let (lead_to_two, two) = make_pair(&listener, "secret");

        let mut lead_peers = vec![lead_to_one, lead_to_two];
        let mut one_peers = vec![one];
        let two_peers = vec![two];

        let (lead_sender, lead_receiver) = channel();
        let (one_sender, one_receiver) = channel();
        let (two_sender, two_receiver) = channel();
        start_readers(&lead_peers, &lead_sender);
        start_readers(&one_peers, &one_sender);
        start_readers(&two_peers, &two_sender);

        send_to_all(&mut one_peers, None, pack("one", "hello everyone").as_str());

        let (index, result) = lead_receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(index, 0);
        let text = result.unwrap();
        assert_eq!(
            unpack(text.as_str()),
            (String::from("one"), String::from("hello everyone"))
        );

        send_to_all(&mut lead_peers, Some(index), text.as_str());

        let (_, result) = two_receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(
            unpack(result.unwrap().as_str()),
            (String::from("one"), String::from("hello everyone"))
        );
        assert!(one_receiver.recv_timeout(Duration::from_millis(300)).is_err());
    }

    #[test]
    fn lead_message_reaches_every_member() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let (lead_to_one, one) = make_pair(&listener, "secret");
        let (lead_to_two, two) = make_pair(&listener, "secret");

        let mut lead_peers = vec![lead_to_one, lead_to_two];
        let one_peers = vec![one];
        let two_peers = vec![two];

        let (one_sender, one_receiver) = channel();
        let (two_sender, two_receiver) = channel();
        start_readers(&one_peers, &one_sender);
        start_readers(&two_peers, &two_sender);

        send_to_all(&mut lead_peers, None, pack("lead", "welcome").as_str());

        let (_, first) = one_receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        let (_, second) = two_receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(unpack(first.unwrap().as_str()).1, "welcome");
        assert_eq!(unpack(second.unwrap().as_str()).1, "welcome");
    }

    #[test]
    fn a_closed_connection_is_reported() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let (lead_to_one, one) = make_pair(&listener, "secret");

        let lead_peers = vec![lead_to_one];
        let mut one_peers = vec![one];

        let (lead_sender, lead_receiver) = channel();
        start_readers(&lead_peers, &lead_sender);

        close_all(&mut one_peers);

        let (index, result) = lead_receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(index, 0);
        assert!(result.is_err());
    }
}
