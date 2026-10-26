use socket2::{Domain, Protocol, Socket, Type};
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;
use std::time::Instant;

const SERVER_PORT: &str = "9000";
pub const LOCAL_PORT: u16 = 9001;

pub fn join_server(
    ip: &str,
    code: &str,
) -> std::result::Result<(String, Vec<TcpStream>), String> {
    let server_address: SocketAddr = match format!("{}:{}", ip, SERVER_PORT).parse() {
        Ok(address) => address,
        Err(err) => {
            return Err(String::from("Wrong server ip: ") + err.to_string().as_str());
        }
    };
    let local_address: SocketAddr = ([0, 0, 0, 0], LOCAL_PORT).into();

    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.set_reuse_port(true).unwrap();
    socket.bind(&local_address.into()).unwrap();
    match socket.connect(&server_address.into()) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt connect to the server: ") + err.to_string().as_str());
        }
    }
    let mut stream: TcpStream = socket.into();

    stream
        .set_read_timeout(Some(Duration::from_secs(150)))
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

    let mut you = String::new();
    let mut lead = String::new();
    for line in peers.lines() {
        if line.starts_with("YOU ") {
            you = String::from(&line[4..]);
        }
        if line.starts_with("LEAD ") {
            lead = String::from(&line[5..]);
        }
    }

    if you == lead {
        let mut handles = Vec::new();
        for line in peers.lines() {
            if line.starts_with("MEMBER ") {
                let member = String::from(&line[7..]);
                let handle = std::thread::spawn(move || connect_peer(LOCAL_PORT, member.as_str()));
                handles.push(handle);
            }
        }

        let total = handles.len();
        let mut streams: Vec<TcpStream> = Vec::new();
        for handle in handles {
            match handle.join().unwrap() {
                Ok(stream) => {
                    streams.push(stream);
                }
                Err(_) => {}
            }
        }
        let message = String::from("Connected to ")
            + streams.len().to_string().as_str()
            + " of "
            + total.to_string().as_str()
            + " members";
        return Ok((message, streams));
    }

    match connect_peer(LOCAL_PORT, lead.as_str()) {
        Ok(stream) => Ok((String::from("Connected to the lead"), vec![stream])),
        Err(message) => Err(message),
    }
}

pub fn connect_peer(local_port: u16, peer: &str) -> std::result::Result<TcpStream, String> {
    let peer_address: SocketAddr = match peer.parse() {
        Ok(address) => address,
        Err(err) => {
            return Err(String::from("Wrong peer address: ") + err.to_string().as_str());
        }
    };
    let local_address: SocketAddr = ([0, 0, 0, 0], local_port).into();
    let deadline = Instant::now() + Duration::from_secs(15);

    while Instant::now() < deadline {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
        socket.set_reuse_address(true).unwrap();
        socket.set_reuse_port(true).unwrap();
        socket.bind(&local_address.into()).unwrap();

        match socket.connect_timeout(&peer_address.into(), Duration::from_secs(1)) {
            Ok(_) => {
                return Ok(socket.into());
            }
            Err(_) => {
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
    Err(String::from("Couldnt connect to the peer"))
}
