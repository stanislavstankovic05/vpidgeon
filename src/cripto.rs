use crate::data::Session;
use sha2::{Digest, Sha256};
use snow::HandshakeState;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::string::*;
use std::time::Duration;

const PROTOCOL: &str = "Noise_NNpsk0_25519_ChaChaPoly_SHA256";
const MAX_MESSAGE_SIZE: usize = 65535;

pub fn send_payload(mut stream: &TcpStream, data: &[u8]) -> std::result::Result<(), String> {
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

pub fn receive_payload(mut stream: &TcpStream) -> std::result::Result<Vec<u8>, String> {
    let mut length_bytes: [u8; 4] = [0; 4];
    match stream.read_exact(&mut length_bytes) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt receive: ") + err.to_string().as_str());
        }
    }
    let length = u32::from_be_bytes(length_bytes) as usize;
    if length > MAX_MESSAGE_SIZE {
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

fn build_handshake(
    shared_key: &str,
    initiator: bool,
) -> std::result::Result<snow::HandshakeState, String> {
    let params: snow::params::NoiseParams = PROTOCOL.parse().unwrap();
    let shared_key_bytes: [u8; 32] = Sha256::digest(shared_key.as_bytes()).into();
    //build the snow builder using the configs
    let builder = match snow::Builder::new(params).psk(0, &shared_key_bytes) {
        Ok(builder) => builder,
        Err(err) => {
            return Err(String::from("Couldnt set the key: ") + err.to_string().as_str());
        }
    };

    let handshake: Result<snow::HandshakeState, snow::Error>;

    if initiator == true {
        handshake = builder.build_initiator();
    } else {
        handshake = builder.build_responder();
    }

    match handshake {
        Ok(handshake) => Ok(handshake),
        Err(err) => Err(String::from("Couldnt start the handshake: ") + err.to_string().as_str()),
    }
}

pub fn run_handshake(
    stream: &mut TcpStream,
    psk: &str,
    initiator: bool,
) -> std::result::Result<Session, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let mut handshake = build_handshake(psk, initiator)?;
    let mut buffer: Vec<u8> = vec![0; MAX_MESSAGE_SIZE];

    if initiator {
        let length = match handshake.write_message(&[], &mut buffer) {
            Ok(length) => length,
            Err(err) => {
                return Err(String::from("Handshake failed: ") + err.to_string().as_str());
            }
        };
        send_payload(stream, &buffer[..length])?;

        let reply = receive_payload(stream)?;
        match handshake.read_message(&reply, &mut buffer) {
            Ok(_) => {}
            Err(err) => {
                return Err(String::from("Handshake failed: ") + err.to_string().as_str());
            }
        }
    } else {
        let message = receive_payload(stream)?;
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
        send_payload(stream, &buffer[..length])?;
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
