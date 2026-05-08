use std::sync::Arc;

use crate::app::ports::{OAuthService, PushService, SupportersService};

pub struct App {
    pub supporters: Arc<dyn SupportersService>,
    pub push: Arc<dyn PushService>,
    pub oauth: Arc<dyn OAuthService>,
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
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            supporters: None,
            push: None,
            oauth: None,
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

    pub fn build(self) -> App {
        App {
            supporters: self.supporters.expect("supporters required"),
            push: self.push.expect("push required"),
            oauth: self.oauth.expect("oauth required"),
        }
    }
}
