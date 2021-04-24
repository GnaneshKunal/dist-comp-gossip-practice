use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use bully::{MembershipList, Port};

#[derive(Debug, Serialize, Deserialize)]
pub struct Gossip {
    hostname: String,
    port: Port,
    pub membership_list: Arc<RwLock<MembershipList>>,
}

impl Gossip {
    pub fn new(hostname: String, port: Port, initial_nodes: &[Port]) -> Self {
        Self {
            hostname,
            port,
            membership_list: Arc::new(RwLock::new(MembershipList::from((
                initial_nodes,
                Some(port),
            )))),
        }
    }

    pub fn get_random(&self) -> Vec<Port> {
        self.membership_list.read().unwrap().get_random()
    }
}
