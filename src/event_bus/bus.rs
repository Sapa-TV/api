use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;

use crate::event_bus::events::{Event, EventDiscriminants};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

pub struct EventBus {
    handlers: Mutex<HashMap<EventDiscriminants, Vec<(HandlerId, Box<dyn ErasedHandler>)>>>,
    tasks: Mutex<JoinSet<()>>,
    next_id: AtomicU64,
}

trait ErasedHandler: Send + Sync {
    fn dispatch(&self, event: Arc<Event>, tasks: &mut JoinSet<()>);
    fn is_once(&self) -> bool {
        false
    }
}

struct Handler<F, Fut>
where
    F: Fn(Arc<Event>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    handler: Arc<F>,
    is_once: bool,
}

impl<F, Fut> Handler<F, Fut>
where
    F: Fn(Arc<Event>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn new(handler: F, is_once: bool) -> Self {
        Self {
            handler: Arc::new(handler),
            is_once,
        }
    }
}

impl<F, Fut> ErasedHandler for Handler<F, Fut>
where
    F: Fn(Arc<Event>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn dispatch(&self, event: Arc<Event>, tasks: &mut JoinSet<()>) {
        let handler = self.handler.clone();
        let event = Arc::clone(&event);
        tasks.spawn(async move {
            handler(event).await;
        });
    }

    fn is_once(&self) -> bool {
        self.is_once
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(HashMap::new()),
            tasks: Mutex::new(JoinSet::new()),
            next_id: AtomicU64::new(0),
        }
    }

    fn next_id(&self) -> HandlerId {
        HandlerId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    fn add_handler<Fut>(
        &self,
        disc: EventDiscriminants,
        handler: impl Fn(Arc<Event>) -> Fut + Send + Sync + 'static,
        is_once: bool,
    ) -> HandlerId
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        let id = self.next_id();
        let handler: Box<dyn ErasedHandler> = Box::new(Handler::new(handler, is_once));

        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(disc)
            .or_insert_with(Vec::new)
            .push((id, handler));
        id
    }

    pub fn on<Fut>(
        &self,
        disc: EventDiscriminants,
        handler: impl Fn(Arc<Event>) -> Fut + Send + Sync + 'static,
    ) -> HandlerId
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.add_handler(disc, handler, false)
    }

    pub fn once<Fut>(
        &self,
        disc: EventDiscriminants,
        handler: impl Fn(Arc<Event>) -> Fut + Send + Sync + 'static,
    ) -> HandlerId
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.add_handler(disc, handler, true)
    }

    pub fn remove_handler(&self, disc: EventDiscriminants, id: HandlerId) {
        let mut handlers = self.handlers.lock().unwrap();
        if let Some(handler_list) = handlers.get_mut(&disc) {
            handler_list.retain(|(h_id, _)| *h_id != id);
        }
    }

    pub fn publish<E: Into<Event>>(&self, event: E) {
        let event = event.into();
        let event = Arc::new(event);
        let discriminant = EventDiscriminants::from(&*event);

        let mut tasks_guard = self.tasks.lock().unwrap();
        let mut handlers = self.handlers.lock().unwrap();

        if let Some(handler_list) = handlers.get_mut(&discriminant) {
            handler_list.retain(|(_, handler)| {
                handler.dispatch(Arc::clone(&event), &mut *tasks_guard);
                !handler.is_once()
            });
        }
    }

    pub async fn wait_for_handlers(&self) {
        let mut tasks_guard = self.tasks.lock().unwrap();
        while let Some(_) = tasks_guard.join_next().await {}
    }

    pub async fn shutdown(&self) {
        // TODO: send shutdown signal to all handlers, await for complete or timeout
        let mut handlers = self.handlers.lock().unwrap();
        let total: usize = handlers.values().map(|v| v.len()).sum();
        if total > 0 {
            tracing::info!("EventBus shutdown: {} handlers still subscribed", total);
        }
        handlers.clear();
        drop(handlers);
        self.wait_for_handlers().await;
        tracing::info!("EventBus: all tasks finished");
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
