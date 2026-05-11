use crate::event_bus::{Event, EventBus, EventDiscriminants};

pub fn register(bus: &EventBus) {
    bus.on(EventDiscriminants::Chat, |event| async move {
        if let Event::Chat(ref msg) = *event.as_ref() {
            tracing::info!(
                "[{:?}] Chat message from {} ({}): {}",
                msg.provider,
                msg.username,
                msg.user_id,
                msg.message
            );
        }
    });
}
