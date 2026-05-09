use std::sync::Arc;

use crate::app::ports::{OAuthService, PushService, SupportersService};
use crate::eventsub::application::EventSubManager;
use crate::push::infra::PushClient;
use crate::token_manager::application::TokenManager;

pub struct App {
    pub supporters: Arc<dyn SupportersService>,
    pub push: Arc<dyn PushService>,
    pub oauth: Arc<dyn OAuthService>,
    pub token_manager: Arc<TokenManager>,
    pub push_client: Arc<PushClient>,
    pub eventsub: Option<Arc<EventSubManager>>,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }
}

pub struct AppBuilder {
    supporters: Option<Arc<dyn SupportersService>>,
    push: Option<Arc<dyn PushService>>,
    oauth: Option<Arc<dyn OAuthService>>,
    token_manager: Option<Arc<TokenManager>>,
    push_client: Option<Arc<PushClient>>,
    eventsub: Option<Arc<EventSubManager>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            supporters: None,
            push: None,
            oauth: None,
            token_manager: None,
            push_client: None,
            eventsub: None,
        }
    }

    pub fn supporters(mut self, s: Arc<dyn SupportersService>) -> Self {
        self.supporters = Some(s);
        self
    }

    pub fn push(mut self, p: Arc<dyn PushService>) -> Self {
        self.push = Some(p);
        self
    }

    pub fn oauth(mut self, o: Arc<dyn OAuthService>) -> Self {
        self.oauth = Some(o);
        self
    }

    pub fn token_manager(mut self, t: Arc<TokenManager>) -> Self {
        self.token_manager = Some(t);
        self
    }

    pub fn push_client(mut self, c: Arc<PushClient>) -> Self {
        self.push_client = Some(c);
        self
    }

    pub fn eventsub(mut self, e: Arc<EventSubManager>) -> Self {
        self.eventsub = Some(e);
        self
    }

    pub fn build(self) -> App {
        App {
            supporters: self.supporters.expect("supporters required"),
            push: self.push.expect("push required"),
            oauth: self.oauth.expect("oauth required"),
            token_manager: self.token_manager.expect("token_manager required"),
            push_client: self.push_client.expect("push_client required"),
            eventsub: self.eventsub,
        }
    }
}
