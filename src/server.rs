/*
 * based on the link on the cloud VM, we start a dummy server to get the public ips.
 *
 * we need to that VM. it will be a link, the user needs to set that link
 */
use dotenvy::*;
use ssh2::*;
use std::env;
use std::net;
use std::net::Ipv4Addr;
use std::net::TcpStream;
use std::path::Path;
use std::thread;

struct VMConnection {
    ip: String,
    name: String,
    sshkey: String,
}

pub fn create_server() -> std::result::Result<String, String> {
    let server_file = match std::fs::read("vm_server.py") {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err(String::from("Couldnt read vm_server.py: ") + err.to_string().as_str());
        }
    };
    let env_file = match std::fs::read("vm/.env") {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err(String::from("Couldnt read vm/.env: ") + err.to_string().as_str());
        }
    };

    let (message, vm_session) = vm_connect()?;

    vm_run(&vm_session, "pkill -f \"[v]m_server.py\"")?;

    vm_upload(&vm_session, "vm_server.py", &server_file, 0o644)?;
    vm_upload(&vm_session, ".env", &env_file, 0o600)?;

    vm_run(
        &vm_session,
        "nohup python3 vm_server.py > server.log 2>&1 < /dev/null &",
    )?;

    let running = vm_run(&vm_session, "sleep 1; pgrep -f \"[v]m_server.py\"")?;
    if running != 0 {
        return Err(String::from("The server did not start, see server.log on the vm"));
    }
    Ok(String::from("Server started on the VM"))
}
pub fn vm_connect() -> std::result::Result<(String, ssh2::Session), String> {
    let filename = ".env";
    dotenvy::from_filename(filename);
    let vm_connection = VMConnection {
        ip: match env::var("VM_IP") {
            Ok(value) => value,
            Err(err) => return Err(String::from("VM_IP missing in .env")),
        },
        name: match env::var("VM_NAME") {
            Ok(value) => value,
            Err(err) => return Err(String::from("VM_NAME missing in .env")),
        },
        sshkey: match env::var("SSH_key") {
            Ok(value) => value,
            Err(err) => return Err(String::from("SSH_key missing in .env")),
        },
    };

    let tcp: TcpStream;
    let try_tcp = match TcpStream::connect(format!("{}:22", vm_connection.ip)) {
        Ok(stream) => tcp = stream,
        Err(err) => {
            return Err(String::from("Couldnt connect to the vm: ") + err.to_string().as_str());
        }
    };

    let mut vm_session = match ssh2::Session::new() {
        Ok(session) => session,
        Err(err) => {
            return Err(String::from("Couldnt start session: ") + err.to_string().as_str());
        }
    };
    vm_session.set_tcp_stream(tcp);
    match vm_session.handshake() {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Handshake failed: ") + err.to_string().as_str());
        }
    }

    let connection_result = vm_session.userauth_pubkey_file(
        &vm_connection.name,
        None,
        Path::new(&vm_connection.sshkey),
        None,
    );
    match connection_result {
        Ok(_) => {
            if !vm_session.authenticated() {
                return Err(String::from("The vm refused the login"));
            }
        }
        Err(err) => {
            return Err(String::from("Couldnt login to the vm: ") + err.to_string().as_str());
        }
    }
    Ok((
        String::from("Connected to the VM as ") + vm_connection.name.as_str(),
        vm_session,
    ))
}

fn vm_upload(
    vm_session: &ssh2::Session,
    remote_name: &str,
    content: &[u8],
    mode: i32,
) -> std::result::Result<(), String> {
    let mut remote_file =
        match vm_session.scp_send(Path::new(remote_name), mode, content.len() as u64, None) {
            Ok(file) => file,
            Err(err) => {
                return Err(String::from("Couldnt upload ")
                    + remote_name
                    + ": "
                    + err.to_string().as_str());
            }
        };
    match std::io::Write::write_all(&mut remote_file, content) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt write ") + remote_name + ": " + err.to_string().as_str());
        }
    }
    let _ = remote_file.send_eof();
    let _ = remote_file.wait_eof();
    let _ = remote_file.close();
    let _ = remote_file.wait_close();
    Ok(())
}

fn vm_run(vm_session: &ssh2::Session, command: &str) -> std::result::Result<i32, String> {
    let mut channel = match vm_session.channel_session() {
        Ok(channel) => channel,
        Err(err) => {
            return Err(String::from("Couldnt open a channel: ") + err.to_string().as_str());
        }
    };
    match channel.exec(command) {
        Ok(_) => {}
        Err(err) => {
            return Err(String::from("Couldnt run the command: ") + err.to_string().as_str());
        }
    }
    let mut output = String::new();
    let _ = std::io::Read::read_to_string(&mut channel, &mut output);
    let _ = channel.wait_close();
    match channel.exit_status() {
        Ok(exit_code) => Ok(exit_code),
        Err(err) => Err(String::from("Couldnt read the exit code: ") + err.to_string().as_str()),
    }
}

pub fn lead_join(
    code: &str,
    psk: &str,
) -> std::result::Result<(String, Vec<crate::data::Peer>), String> {
    dotenvy::from_filename(".env");
    let ip = match env::var("VM_IP") {
        Ok(value) => value,
        Err(err) => return Err(String::from("VM_IP missing in .env")),
    };
    crate::client::join_server(ip.as_str(), code, psk)
}

pub fn vm_configured() -> bool {
    dotenvy::from_filename(".env");
    let has_ip = env::var("VM_IP").is_ok();
    let has_name = env::var("VM_NAME").is_ok();
    let has_key = env::var("SSH_key").is_ok();
    has_ip && has_name && has_key
}

pub fn lead_direct(
    port: u16,
    psk: &str,
    max_members: usize,
    seconds: u64,
) -> std::result::Result<(String, Vec<crate::data::Peer>), String> {
    let listener = match std::net::TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => listener,
        Err(err) => {
            return Err(String::from("Couldnt listen on the port: ") + err.to_string().as_str());
        }
    };
    listener.set_nonblocking(true).unwrap();

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut peers: Vec<crate::data::Peer> = Vec::new();
    let mut pending = 0;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);

    while std::time::Instant::now() < deadline && peers.len() < max_members {
        if peers.len() + pending < max_members {
            match listener.accept() {
                Ok((stream, address)) => {
                    pending = pending + 1;
                    let sender = sender.clone();
                    let psk = String::from(psk);
                    std::thread::spawn(move || {
                        let mut stream = stream;
                        stream.set_nonblocking(false).unwrap();
                        let result = match crate::cripto::handshake_lead(&mut stream, psk.as_str()) {
                            Ok(session) => Ok(crate::data::Peer {
                                address: address.to_string(),
                                stream: stream,
                                session: std::sync::Arc::new(std::sync::Mutex::new(session)),
                            }),
                            Err(message) => Err(message),
                        };
                        let _ = sender.send(result);
                    });
                }
                Err(_) => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));

        loop {
            match receiver.try_recv() {
                Ok(Ok(peer)) => {
                    peers.push(peer);
                    pending = pending - 1;
                }
                Ok(Err(_)) => {
                    pending = pending - 1;
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    let message = String::from("Secure connection with ")
        + peers.len().to_string().as_str()
        + " members";
    Ok((message, peers))
}
