pub mod chat;
pub mod stream;

pub fn register(bus: &crate::event_bus::EventBus) {
    chat::register(bus);
    stream::register(bus);
}
