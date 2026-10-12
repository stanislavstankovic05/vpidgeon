use std::net;
use std::net::Ipv4Addr;
use std::thread;

use crate::data::User;
mod cripto;
pub mod data;
mod network;
mod tui;

fn main() {
    let mut user: data::User = User {
        private_ip: (Ipv4Addr::new(0, 0, 0, 0)),
        public_ip: (Ipv4Addr::new(0, 0, 0, 0)),
    };

    user.instance_user();
}
