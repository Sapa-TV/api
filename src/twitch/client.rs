use std::sync::Arc;

use crate::db::Db;
use crate::error::AppResult;
use crate::twitch::auth::UserTokenManager;

#[allow(dead_code)]
pub struct TwitchClient {
    pub auth: Arc<UserTokenManager>,
}

#[allow(dead_code)]
impl TwitchClient {
    pub fn new(auth: UserTokenManager) -> Self {
        Self {
            auth: Arc::new(auth),
        }
    }

    pub async fn load_token<T: Db + Send + Sync>(&self, db: &T) -> AppResult<()> {
        self.auth.load_from_db(db).await
    }

    pub fn auth(&self) -> Arc<UserTokenManager> {
        Arc::clone(&self.auth)
    }
}
