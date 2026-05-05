use crate::error::AppResult;

#[async_trait::async_trait]
pub trait AdminWhiteListRepository {
    async fn is_admin_whitelisted(&self, twitch_id: &str) -> AppResult<bool>;
    async fn add_admin_to_whitelist(
        &self,
        twitch_id: &str,
        username: Option<&str>,
        added_by: Option<&str>,
    ) -> AppResult<()>;
    async fn remove_admin_from_whitelist(&self, twitch_id: &str) -> AppResult<bool>;
    async fn list_admin_whitelist(&self) -> AppResult<Vec<(String, Option<String>)>>;
}
