#![feature(btree_retain)]

#[macro_use]
extern crate log;

pub mod beatcount;
pub mod timestamp;
pub mod utils;

use std::{
    collections::BTreeMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};

use beatcount::BeatCount;
use utils::get_timestamp;

pub type Port = u16;

pub const SWEEP_TIME: i64 = 5;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MembershipList(pub BTreeMap<Port, BeatCount>, pub Option<Port>);

impl MembershipList {
    pub fn new(members: BTreeMap<Port, BeatCount>, port: Option<Port>) -> Self {
        Self(members, port)
    }

    pub fn get_random(&self) -> Vec<Port> {
        let mut rng = thread_rng();
        self.0
            .iter()
            .filter(|(k, _)| self.1.map_or(true, |p| p != **k))
            .choose_multiple(&mut rng, 2)
            .iter()
            .map(|kv| *kv.0)
            .collect()
    }

    pub fn merge(&mut self, members: &MembershipList, member: SocketAddr) {
        for (port, BeatCount(count, time)) in &members.0 {
            match self.0.get_mut(port) {
                Some(ref mut beatcount) => {
                    if *count > beatcount.get_count() {
                        beatcount.0 = *count;
                        beatcount.just_touch();
                    }

                    if member.port() == *port {
                        beatcount.increment_and_touch();
                    }
                }
                None => {
                    if (get_timestamp() - time.get_timestamp()) < SWEEP_TIME {
                        info!("New node: {}", *port);
                        self.0
                            .insert(*port, BeatCount::from((*count, time.get_timestamp())));
                        info!("Memberlist: {:?}", self);
                    } else {
                        info!(
                            "Tried to add expired node: {:?} from: {:?}",
                            port,
                            member.port()
                        );
                    }
                }
            }
        }

        self.sweep();
    }

    pub fn sweep(&mut self) {
        let member_port = self.1.clone();
        self.0.retain(|port, bc| {
            if (get_timestamp() - bc.get_timestamp()) > SWEEP_TIME
                && member_port.map_or(true, |p| p != *port)
            {
                warn!("Removing {:?} - {:?}", port, bc);
                return false;
            }
            return true;
        });
    }
}

impl From<(BTreeMap<Port, BeatCount>, Option<Port>)> for MembershipList {
    fn from(members_port: (BTreeMap<Port, BeatCount>, Option<Port>)) -> Self {
        Self::new(members_port.0, members_port.1)
    }
}

impl From<(&[Port], Option<Port>)> for MembershipList {
    fn from(members_port: (&[Port], Option<Port>)) -> Self {
        MembershipList::from((
            members_port
                .0
                .iter()
                .map(|p| (*p, BeatCount::new()))
                .collect::<BTreeMap<Port, BeatCount>>(),
            members_port.1,
        ))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Message {
    MemList(MembershipList),
    Data(String),
    Error(String),
}

pub fn transmit_membership_list(node: &UdpSocket, membership_list: &MembershipList, to_node: Port) {
    let data = Message::MemList((*membership_list).clone());

    let membership_list_encoded: Vec<u8> = bincode::serialize(&data).unwrap();

    node.send_to(&membership_list_encoded, format!("localhost:{}", to_node))
        .unwrap();
}

pub fn receive_membership_list(node: UdpSocket) -> (Message, Option<SocketAddr>) {
    let mut buf = [0u8; 1024];
    node.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

    match node.recv_from(&mut buf) {
        Ok((_amt, src_addr)) => {
            let message: Message = bincode::deserialize(&buf).unwrap();
            // info!("Received from {:?} and data is {:?}", src_addr, message);
            (message, Some(src_addr))
        }
        Err(err) => (Message::Error(format!("Error: {:?}", err)), None),
    }
}

pub fn transmitter_thread(
    socket: &UdpSocket,
    membership_list: &Arc<RwLock<MembershipList>>,
    sleep_interval: Duration,
) {
    thread::sleep(sleep_interval);
    let to_nodes = membership_list.read().unwrap().get_random();
    for to_node in &to_nodes {
        let membership_list = membership_list.clone();
        transmit_membership_list(&socket, &membership_list.read().unwrap(), *to_node);
    }
}

pub fn process_message(
    message: Message,
    membership_list: Arc<RwLock<MembershipList>>,
    src_addr: Option<SocketAddr>,
) {
    match message {
        Message::MemList(members) => {
            // info!("Got members: {:?}", members);
            membership_list
                .write()
                .unwrap()
                .merge(&members, src_addr.unwrap())
        }
        Message::Data(string) => info!("{}", string),
        Message::Error(err) => debug!("{}", err),
    }
}
