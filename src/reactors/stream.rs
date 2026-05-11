use crate::event_bus::{Event, EventBus, EventDiscriminants};

pub fn register(bus: &EventBus) {
    bus.on(EventDiscriminants::Stream, |event| async move {
        if let Event::Stream(ref evt) = *event.as_ref() {
            tracing::info!(
                "[{:?}] Stream {:?} at {}",
                evt.provider,
                evt.status,
                evt.timestamp
            );
        }
    });
}
