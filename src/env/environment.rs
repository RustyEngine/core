use crate::network::packet::NetworkEventHandler;
use crate::network::packet::NetworkHandler;

pub struct Environment {
}

impl NetworkEventHandler for Environment {
    fn on_client_add(&self, id: usize, network_handler: &NetworkHandler) -> bool {
        true
    }

    fn on_client_remove(&self, id: usize, network_handler: &NetworkHandler) {
        
    }
}