use std::collections::HashMap;
use tokio::sync::mpsc;
use warp::ws::Message;

pub trait NetworkEventHandler {
    fn on_client_add(&self, id: usize, network_handler: &NetworkHandler) -> bool;

    fn on_client_remove(&self, id: usize, network_handler: &NetworkHandler);
}

pub struct NetworkHandler {
    next_client_id: usize,
    pub clients: HashMap<usize, mpsc::UnboundedSender<Message>>,
    pub sender: mpsc::UnboundedSender<(usize, Message)>,
    receiver: mpsc::UnboundedReceiver<(usize, Message)>,
    event_handler: Box<dyn NetworkEventHandler + Send>
}

impl NetworkHandler {
    pub fn new(event_handler: Box<dyn NetworkEventHandler + Send>) -> NetworkHandler {
        let (tx, rx) = mpsc::unbounded_channel::<(usize, Message)>(); // ID, message
        NetworkHandler {
            next_client_id: 0,
            clients: HashMap::new(),
            sender: tx,
            receiver: rx,
            event_handler
        }
    }

    pub fn flush(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
        
        }
    }

    pub fn add_client(&mut self, sender: mpsc::UnboundedSender<Message>) -> Option<usize> {
        if !self.event_handler.on_client_add(self.next_client_id, &*self) {
            return None;
        }

        let id = self.next_client_id;
        self.next_client_id += 1;
        self.clients.insert(id, sender);
        Some(id)
    }

    pub fn remove_client(&mut self, id: usize) {
        self.clients.remove(&id);
        self.event_handler.on_client_remove(id, &*self);
    }

    pub fn get_sender(&self) -> mpsc::UnboundedSender<(usize, Message)> {
        self.sender.clone()
    }
}