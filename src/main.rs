extern crate log;

use std::{net::UdpSocket, thread, time::Duration};

pub mod gossip;
use bully::{process_message, receive_membership_list, transmitter_thread, Port};
use gossip::Gossip;

fn start_transmitter_thread(
    socket: &UdpSocket,
    gossip: &Gossip,
    sleep_interval: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn({
        let socket = socket.try_clone().unwrap();
        let membership_list = gossip.membership_list.clone();
        move || loop {
            transmitter_thread(&socket, &membership_list, sleep_interval);
        }
    })
}

fn start_receiver_thread(socket: &UdpSocket, gossip: &Gossip) -> thread::JoinHandle<()> {
    thread::spawn({
        let membership_list = gossip.membership_list.clone();
        let socket = socket.try_clone().unwrap();
        move || loop {
            let membership_list = membership_list.clone();
            let socket = socket.try_clone().unwrap();
            let (message, src_addr) = receive_membership_list(socket);
            process_message(message, membership_list, src_addr);
        }
    })
}

fn main() {
    env_logger::init();

    let args: Vec<Port> = std::env::args()
        .skip(1)
        .map(|n| n.parse().unwrap())
        .collect();

    let port = &args[0];

    let hostname = "localhost".to_string();
    let gossip = gossip::Gossip::new(hostname.clone(), *port, &args[0..]);

    let socket = UdpSocket::bind(format!("{}:{}", hostname, port)).unwrap();

    let handle1 = start_transmitter_thread(&socket, &gossip, Duration::from_secs(2));

    let handle2 = start_receiver_thread(&socket, &gossip);

    let handle3 = thread::spawn({
        let membership_list = gossip.membership_list.clone();
        move || loop {
            thread::sleep(Duration::from_secs(5));
            membership_list.write().unwrap().sweep();
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
}
