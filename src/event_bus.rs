use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait EventBus {
    type Event: Send + Sync + 'static;
    fn publish(&self, event: Self::Event);
    fn subscribe(&self) -> Box<dyn Iterator<Item = Self::Event> + Send>;
}

#[derive(Clone)]
pub struct InMemoryBus<E: Send + Sync + Clone + 'static> {
    queues: Arc<Mutex<HashMap<String, Vec<E>>>>,
}

impl<E: Send + Sync + Clone + 'static> InMemoryBus<E> {
    pub fn new() -> Self {
        Self { queues: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl<E: Send + Sync + Clone + 'static> EventBus for InMemoryBus<E> {
    type Event = E;

    fn publish(&self, event: Self::Event) {
        let mut q = self.queues.lock().unwrap();
        let entry = q.entry("default".to_string()).or_insert_with(Vec::new);
        entry.push(event);
    }

    fn subscribe(&self) -> Box<dyn Iterator<Item = Self::Event> + Send> {
        let cloned = self.queues.lock().unwrap().get("default").cloned().unwrap_or_default();
        Box::new(cloned.into_iter())
    }
}
