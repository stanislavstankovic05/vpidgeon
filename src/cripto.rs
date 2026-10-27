use sha2::{Digest, Sha256};
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use std::string::*;

const PROTOCOL: &str = "Noise_NNpsk0_25519_ChaChaPoly_SHA256";
const MAX_FRAME: usize = 65535;

pub struct Session {
    transport: snow::TransportState,
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

pub fn write_frame(stream: &mut TcpStream, data: &[u8]) -> std::result::Result<(), String> {
    let length = (data.len() as u32).to_be_bytes();
    match stream.write_all(&length) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt send: ") + err.to_string().as_str());
        }
    }
    match stream.write_all(data) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt send: ") + err.to_string().as_str());
        }
    }
    Ok(())
}

pub fn read_frame(stream: &mut TcpStream) -> std::result::Result<Vec<u8>, String> {
    let mut length_bytes: [u8; 4] = [0; 4];
    match stream.read_exact(&mut length_bytes) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt receive: ") + err.to_string().as_str());
        }
    }
    let length = u32::from_be_bytes(length_bytes) as usize;
    if length > MAX_FRAME {
        return Err(String::from("The message is too big"));
    }
    let mut data: Vec<u8> = vec![0; length];
    match stream.read_exact(&mut data) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt receive: ") + err.to_string().as_str());
        }
    }
    Ok(data)
}

fn build_handshake(psk: &str, initiator: bool) -> std::result::Result<snow::HandshakeState, String> {
    let params: snow::params::NoiseParams = PROTOCOL.parse().unwrap();
    let psk_bytes: [u8; 32] = Sha256::digest(psk.as_bytes()).into();
    let builder = match snow::Builder::new(params).psk(0, &psk_bytes) {
        Ok(builder) => builder,
        Err(err) => {
            return Err(String::from("Couldnt set the key: ") + err.to_string().as_str());
        }
    };
    let handshake = if initiator {
        builder.build_initiator()
    } else {
        builder.build_responder()
    };
    match handshake {
        Ok(handshake) => Ok(handshake),
        Err(err) => Err(String::from("Couldnt start the handshake: ") + err.to_string().as_str()),
    }
}

pub fn handshake_member(stream: &mut TcpStream, psk: &str) -> std::result::Result<Session, String> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut handshake = build_handshake(psk, true)?;
    let mut buffer: Vec<u8> = vec![0; MAX_FRAME];

    let length = match handshake.write_message(&[], &mut buffer) {
        Ok(length) => length,
        Err(err) => {
            return Err(String::from("Handshake failed: ") + err.to_string().as_str());
        }
    };
    write_frame(stream, &buffer[..length])?;

    let reply = read_frame(stream)?;
    match handshake.read_message(&reply, &mut buffer) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Handshake failed: ") + err.to_string().as_str());
        }
    }

    let transport = match handshake.into_transport_mode() {
        Ok(transport) => transport,
        Err(err) => {
            return Err(String::from("Handshake failed: ") + err.to_string().as_str());
        }
    };
    stream.set_read_timeout(None).unwrap();
    Ok(Session { transport })
}

pub fn handshake_lead(stream: &mut TcpStream, psk: &str) -> std::result::Result<Session, String> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut handshake = build_handshake(psk, false)?;
    let mut buffer: Vec<u8> = vec![0; MAX_FRAME];

    let message = read_frame(stream)?;
    match handshake.read_message(&message, &mut buffer) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Handshake failed: ") + err.to_string().as_str());
        }
    }

    let length = match handshake.write_message(&[], &mut buffer) {
        Ok(length) => length,
        Err(err) => {
            return Err(String::from("Handshake failed: ") + err.to_string().as_str());
        }
    };
    write_frame(stream, &buffer[..length])?;

    let transport = match handshake.into_transport_mode() {
        Ok(transport) => transport,
        Err(err) => {
            return Err(String::from("Handshake failed: ") + err.to_string().as_str());
        }
    };
    stream.set_read_timeout(None).unwrap();
    Ok(Session { transport })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

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
            let mut session = handshake_lead(&mut lead_stream, "secret").unwrap();
            let frame = read_frame(&mut lead_stream).unwrap();
            let received = session.decrypt(&frame).unwrap();
            let reply = session.encrypt(b"hello member").unwrap();
            write_frame(&mut lead_stream, &reply).unwrap();
            received
        });

        let mut session = handshake_member(&mut member_stream, "secret").unwrap();
        let data = session.encrypt(b"hello lead").unwrap();
        write_frame(&mut member_stream, &data).unwrap();
        let frame = read_frame(&mut member_stream).unwrap();
        let answer = session.decrypt(&frame).unwrap();

        assert_eq!(answer, b"hello member");
        assert_eq!(lead_thread.join().unwrap(), b"hello lead");
    }

    #[test]
    fn different_psk_fails() {
        let (mut member_stream, mut lead_stream) = connected_pair();
        let lead_thread =
            std::thread::spawn(move || handshake_lead(&mut lead_stream, "lead secret").is_err());

        let member_failed = handshake_member(&mut member_stream, "other secret").is_err();

        assert!(member_failed);
        assert!(lead_thread.join().unwrap());
    }

    #[test]
    fn tampered_and_replayed_messages_fail() {
        let (mut member_stream, mut lead_stream) = connected_pair();
        let lead_thread = std::thread::spawn(move || {
            let mut session = handshake_lead(&mut lead_stream, "secret").unwrap();
            let first = read_frame(&mut lead_stream).unwrap();
            let good = session.decrypt(&first).is_ok();
            let replayed = session.decrypt(&first).is_ok();
            let second = read_frame(&mut lead_stream).unwrap();
            let tampered = session.decrypt(&second).is_ok();
            (good, replayed, tampered)
        });

        let mut session = handshake_member(&mut member_stream, "secret").unwrap();
        let first = session.encrypt(b"one").unwrap();
        write_frame(&mut member_stream, &first).unwrap();
        let mut second = session.encrypt(b"two").unwrap();
        second[0] = second[0] ^ 1;
        write_frame(&mut member_stream, &second).unwrap();

        let (good, replayed, tampered) = lead_thread.join().unwrap();
        assert!(good);
        assert!(!replayed);
        assert!(!tampered);
    }
}
