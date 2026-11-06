use crate::chat::{close_all, pack, send_to, send_to_all, start_readers, unpack, MAX_MESSAGE};
use crate::cripto::run_handshake;
use crate::data::Connection;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

fn make_pair() -> (Connection, Connection) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let mut member_stream = TcpStream::connect(address).unwrap();
    let (mut lead_stream, _) = listener.accept().unwrap();

    let lead_thread = std::thread::spawn(move || {
        let session = run_handshake(&mut lead_stream, "secret", false).unwrap();
        Connection::new(lead_stream, session)
    });
    let session = run_handshake(&mut member_stream, "secret", true).unwrap();
    let member = Connection::new(member_stream, session);
    (lead_thread.join().unwrap(), member)
}

#[test]
fn too_long_message_is_refused() {
    let (lead_side, _member) = make_pair();
    let long = "x".repeat(MAX_MESSAGE + 1);
    assert!(send_to(&lead_side, "a", long.as_str()).is_err());
    let longest = "x".repeat(MAX_MESSAGE);
    assert!(send_to(&lead_side, "a", longest.as_str()).is_ok());
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
    assert_eq!(
        member_1_receiver.recv_timeout(wait).unwrap().content,
        "welcome"
    );
    assert_eq!(
        member_2_receiver.recv_timeout(wait).unwrap().content,
        "welcome"
    );

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
