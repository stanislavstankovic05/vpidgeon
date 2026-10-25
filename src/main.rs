use std::net;
use std::net::Ipv4Addr;
use std::thread;

use crate::data::User;
use crate::tui::App;
mod client;
mod cripto;
pub mod data;
mod network;
mod server;
mod tui;
fn main() {
    // let mut user: data::User = User {
    //     private_ip: (Ipv4Addr::new(0, 0, 0, 0)),
    //     public_ip: (Ipv4Addr::new(0, 0, 0, 0)),
    // };

    //    user.instance_user();

    ratatui::run(|terminal| App::inst().run(terminal));
}
