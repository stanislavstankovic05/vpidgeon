use crate::cripto::run_handshake;
use crate::data::Connection;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

const SERVER_PORT: &str = "9000";

pub fn join_server(
    ip: &str,
    code: &str,
    psk: &str,
) -> std::result::Result<(String, Vec<Connection>), String> {
    let mut stream = match TcpStream::connect(format!("{}:{}", ip, SERVER_PORT)) {
        Ok(stream) => stream,
        Err(err) => {
            return Err(String::from("Couldnt connect to the server: ") + err.to_string().as_str());
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(310)))
        .unwrap();

    let message = String::from(code) + "\n";
    stream.write_all(message.as_bytes()).unwrap();

    let mut peers = String::new();
    match stream.read_to_string(&mut peers) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt read the peers: ") + err.to_string().as_str());
        }
    }

    if peers.is_empty() {
        return Err(String::from(
            "Wrong password, or the server closed without a list",
        ));
    }

    match std::fs::write("peers.txt", &peers) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt save peers.txt: ") + err.to_string().as_str());
        }
    }

    drop(stream);

    let mut self_peer = String::new();
    let mut host_peer = String::new();
    for line in peers.lines() {
        if line.starts_with("YOU ") {
            self_peer = String::from(&line[4..]);
        }
        if line.starts_with("LEAD ") {
            host_peer = String::from(&line[5..]);
        }
    }

    if self_peer == host_peer {
        let mut handles = Vec::new();
        let mut index = 0;
        for line in peers.lines() {
            if line.starts_with("MEMBER ") {
                index = index + 1;
                let server_ip = String::from(ip);
                let server_code = String::from(code);
                let secret = String::from(psk);
                let handle = std::thread::spawn(move || -> std::result::Result<Connection, String> {
                    let mut stream = match connect_server(
                        server_ip.as_str(),
                        server_code.as_str(),
                        "LEAD",
                        index,
                    ) {
                        Ok(stream) => stream,
                        Err(message) => {
                            return Err(message);
                        }
                    };
                    let session = match run_handshake(&mut stream, secret.as_str(), false) {
                        Ok(session) => session,
                        Err(message) => {
                            return Err(message);
                        }
                    };
                    Ok(Connection::new(stream, session))
                });
                handles.push(handle);
            }
        }

        let total = handles.len();
        let mut connected: Vec<Connection> = Vec::new();
        let mut errors = String::new();
        for handle in handles {
            match handle.join().unwrap() {
                Ok(peer) => {
                    connected.push(peer);
                }
                Err(message) => {
                    errors = errors + message.as_str() + ". ";
                }
            }
        }
        let message = String::from("Connected to ")
            + connected.len().to_string().as_str()
            + " of "
            + total.to_string().as_str()
            + " members. "
            + errors.as_str();
        return Ok((message, connected));
    }

    let mut index = 0;
    let mut my_index = 0;
    for line in peers.lines() {
        if line.starts_with("MEMBER ") {
            index = index + 1;
            if &line[7..] == self_peer.as_str() {
                my_index = index;
            }
        }
    }
    if my_index == 0 {
        return Err(String::from(
            "Your address is not in the list from the server",
        ));
    }

    let mut stream = match connect_server(ip, code, "MEMBER", my_index) {
        Ok(stream) => stream,
        Err(message) => {
            return Err(message);
        }
    };
    let session = match run_handshake(&mut stream, psk, true) {
        Ok(session) => session,
        Err(message) => {
            return Err(message);
        }
    };
    let peer = Connection::new(stream, session);
    Ok((String::from("Connected to the lead"), vec![peer]))
}

pub fn connect_server(
    ip: &str,
    code: &str,
    role: &str,
    index: usize,
) -> std::result::Result<TcpStream, String> {
    let server_address: SocketAddr = match format!("{}:{}", ip, SERVER_PORT).parse() {
        Ok(address) => address,
        Err(err) => {
            return Err(String::from("Wrong server ip: ") + err.to_string().as_str());
        }
    };
    let mut stream = match TcpStream::connect_timeout(&server_address, Duration::from_secs(10)) {
        Ok(stream) => stream,
        Err(err) => {
            return Err(String::from("Couldnt connect to the server: ") + err.to_string().as_str());
        }
    };
    stream.set_nodelay(true).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .unwrap();

    let connect_to_vm_test = format!("RELAY {} {} {}\n", code, role, index);
    match stream.write_all(connect_to_vm_test.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt write to the server: ") + err.to_string().as_str());
        }
    }

    let mut answer = String::new();
    let mut byte = [0u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) => {
                return Err(String::from("The server closed before the other peer came"));
            }
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                answer.push(byte[0] as char);
            }
            Err(err) => {
                return Err(String::from("The other peer did not reach the server: ")
                    + err.to_string().as_str());
            }
        }
    }
    if answer != "PAIRED" {
        return Err(String::from("Wrong answer from the server: ") + answer.as_str());
    }

    stream.set_read_timeout(None).unwrap();
    Ok(stream)
}
