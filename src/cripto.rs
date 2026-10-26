use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use x25519_dalek::{EphemeralSecret, PublicKey};
use zeroize::Zeroize;
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

const PROTOCOL: &str = "vpidgeon v1 X25519 HKDF-SHA256 ChaCha20Poly1305";
const MAX_FRAME: usize = 65535;

pub struct Session {
    send_key: [u8; 32],
    receive_key: [u8; 32],
    send_counter: u64,
    receive_counter: u64,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.send_key.zeroize();
        self.receive_key.zeroize();
    }
}

fn make_nonce(counter: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[4..].copy_from_slice(&counter.to_be_bytes());
    nonce
}

impl Session {
    pub fn encrypt(&mut self, plaintext: &[u8]) -> std::result::Result<Vec<u8>, String> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.send_key));
        let nonce = make_nonce(self.send_counter);
        match cipher.encrypt(Nonce::from_slice(&nonce), plaintext) {
            Ok(ciphertext) => {
                self.send_counter = self.send_counter + 1;
                Ok(ciphertext)
            }
            Err(_) => Err(String::from("Couldnt encrypt")),
        }
    }

    pub fn decrypt(&mut self, ciphertext: &[u8]) -> std::result::Result<Vec<u8>, String> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.receive_key));
        let nonce = make_nonce(self.receive_counter);
        match cipher.decrypt(Nonce::from_slice(&nonce), ciphertext) {
            Ok(plaintext) => {
                self.receive_counter = self.receive_counter + 1;
                Ok(plaintext)
            }
            Err(_) => Err(String::from("Couldnt decrypt, the message is not valid")),
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
    let mut length_bytes = [0u8; 4];
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
    let mut data = vec![0u8; length];
    match stream.read_exact(&mut data) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt receive: ") + err.to_string().as_str());
        }
    }
    Ok(data)
}

fn new_hash() -> [u8; 32] {
    Sha256::digest(PROTOCOL.as_bytes()).into()
}

fn mix_hash(hash: [u8; 32], message: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(hash);
    hasher.update(message);
    hasher.finalize().into()
}

fn derive_keys(psk: &str, dh: &[u8; 32], hash: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hkdf = Hkdf::<Sha256>::new(Some(psk.as_bytes()), dh);
    let mut okm = [0u8; 64];
    hkdf.expand(hash, &mut okm).unwrap();

    let mut member_to_lead = [0u8; 32];
    let mut lead_to_member = [0u8; 32];
    member_to_lead.copy_from_slice(&okm[..32]);
    lead_to_member.copy_from_slice(&okm[32..]);
    okm.zeroize();
    (member_to_lead, lead_to_member)
}

fn make_message(public: &PublicKey) -> [u8; 48] {
    let mut message = [0u8; 48];
    message[..32].copy_from_slice(public.as_bytes());
    OsRng.fill_bytes(&mut message[32..]);
    message
}

fn read_public(message: &[u8]) -> std::result::Result<PublicKey, String> {
    if message.len() != 48 {
        return Err(String::from("Handshake failed: wrong message size"));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&message[..32]);
    Ok(PublicKey::from(bytes))
}

pub fn handshake_member(stream: &mut TcpStream, psk: &str) -> std::result::Result<Session, String> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();

    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    let first = make_message(&public);
    write_frame(stream, &first)?;

    let second = read_frame(stream)?;
    let lead_public = read_public(&second)?;

    let mut hash = new_hash();
    hash = mix_hash(hash, &first);
    hash = mix_hash(hash, &second);

    let shared = secret.diffie_hellman(&lead_public);
    if !shared.was_contributory() {
        return Err(String::from("Handshake failed: bad public key"));
    }
    let (send_key, receive_key) = derive_keys(psk, shared.as_bytes(), &hash);
    let mut session = Session {
        send_key,
        receive_key,
        send_counter: 0,
        receive_counter: 0,
    };

    let hello = session.encrypt(b"member hello")?;
    write_frame(stream, &hello)?;
    let answer = read_frame(stream)?;
    let answer = session.decrypt(&answer)?;
    if answer != b"lead hello" {
        return Err(String::from("Handshake failed: wrong answer"));
    }

    stream.set_read_timeout(None).unwrap();
    Ok(session)
}

pub fn handshake_lead(stream: &mut TcpStream, psk: &str) -> std::result::Result<Session, String> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();

    let first = read_frame(stream)?;
    let member_public = read_public(&first)?;

    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    let second = make_message(&public);
    write_frame(stream, &second)?;

    let mut hash = new_hash();
    hash = mix_hash(hash, &first);
    hash = mix_hash(hash, &second);

    let shared = secret.diffie_hellman(&member_public);
    if !shared.was_contributory() {
        return Err(String::from("Handshake failed: bad public key"));
    }
    let (receive_key, send_key) = derive_keys(psk, shared.as_bytes(), &hash);
    let mut session = Session {
        send_key,
        receive_key,
        send_counter: 0,
        receive_counter: 0,
    };

    let hello = read_frame(stream)?;
    let hello = session.decrypt(&hello)?;
    if hello != b"member hello" {
        return Err(String::from("Handshake failed: wrong answer"));
    }
    let answer = session.encrypt(b"lead hello")?;
    write_frame(stream, &answer)?;

    stream.set_read_timeout(None).unwrap();
    Ok(session)
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
