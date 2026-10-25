use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

const SERVER_PORT: &str = "9000";

pub fn join_server(ip: &str, code: &str) -> std::result::Result<String, String> {
    let mut stream = match TcpStream::connect(format!("{}:{}", ip, SERVER_PORT)) {
        Ok(stream) => stream,
        Err(err) => {
            return Err(String::from("Couldnt connect to the server: ") + err.to_string().as_str());
        }
    };

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
        return Err(String::from("Wrong password, or the server closed without a list"));
    }

    match std::fs::write("peers.txt", peers) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt save peers.txt: ") + err.to_string().as_str());
        }
    }
    Ok(String::from("Peers saved in peers.txt"))
}
