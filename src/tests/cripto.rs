use crate::cripto::{receive_payload, run_handshake, send_payload};
use std::net::{TcpListener, TcpStream};

fn connected_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let member = TcpStream::connect(address).unwrap();
    let (lead, _) = listener.accept().unwrap();
    (member, lead)
}

#[test]
fn same_psk_works_both_ways() {
    let (mut member_stream, mut lead_stream) = connected_pair();
    let lead_thread = std::thread::spawn(move || {
        let mut session = run_handshake(&mut lead_stream, "secret", false).unwrap();
        let message = receive_payload(&mut lead_stream).unwrap();
        let received = session.decrypt(&message).unwrap();
        let reply = session.encrypt(b"hello member").unwrap();
        send_payload(&mut lead_stream, &reply).unwrap();
        received
    });

    let mut session = run_handshake(&mut member_stream, "secret", true).unwrap();
    let data = session.encrypt(b"hello lead").unwrap();
    send_payload(&mut member_stream, &data).unwrap();
    let message = receive_payload(&mut member_stream).unwrap();
    let answer = session.decrypt(&message).unwrap();

    assert_eq!(answer, b"hello member");
    assert_eq!(lead_thread.join().unwrap(), b"hello lead");
}

#[test]
fn different_psk_fails() {
    let (mut member_stream, mut lead_stream) = connected_pair();
    let lead_thread = std::thread::spawn(move || {
        run_handshake(&mut lead_stream, "lead secret", false).is_err()
    });

    let member_failed = run_handshake(&mut member_stream, "other secret", true).is_err();

    assert!(member_failed);
    assert!(lead_thread.join().unwrap());
}

#[test]
fn tampered_and_replayed_messages_fail() {
    let (mut member_stream, mut lead_stream) = connected_pair();
    let lead_thread = std::thread::spawn(move || {
        let mut session = run_handshake(&mut lead_stream, "secret", false).unwrap();
        let first = receive_payload(&mut lead_stream).unwrap();
        let good = session.decrypt(&first).is_ok();
        let replayed = session.decrypt(&first).is_ok();
        let second = receive_payload(&mut lead_stream).unwrap();
        let tampered = session.decrypt(&second).is_ok();
        (good, replayed, tampered)
    });

    let mut session = run_handshake(&mut member_stream, "secret", true).unwrap();
    let first = session.encrypt(b"one").unwrap();
    send_payload(&mut member_stream, &first).unwrap();
    let mut second = session.encrypt(b"two").unwrap();
    second[0] = second[0] ^ 1;
    send_payload(&mut member_stream, &second).unwrap();

    let (good, replayed, tampered) = lead_thread.join().unwrap();
    assert!(good);
    assert!(!replayed);
    assert!(!tampered);
}
