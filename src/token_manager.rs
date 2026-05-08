use std::{collections::HashMap, sync::Arc};
use twitch_api::twitch_oauth2::UserToken;

use crate::{
    error::AppResult,
    providers::token_repository::{AccountVariant, ProviderVariant, TokenRecord, TokenRepository},
};
pub use token_provider::*;

mod token_provider;

#[derive(Debug, Clone)]
pub enum TokenEnum {
    Twitch(UserToken),
}

pub struct TokenManagerS<R> {
    repo: R,
    tokens: HashMap<(ProviderVariant, AccountVariant), TokenRecord>,
    providers: HashMap<ProviderVariant, Arc<dyn TokenProvider>>,
}

#[async_trait::async_trait]
trait TokenManager<R> {
    fn new(repo: R) -> Self;
    fn register_provider(&mut self, variant: ProviderVariant, provider: Arc<dyn TokenProvider>);
    fn get_provider(&self, variant: ProviderVariant) -> Option<Arc<dyn TokenProvider>>;
    fn token(&self, provider: ProviderVariant, account: AccountVariant) -> Option<&TokenRecord>;
    async fn get_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord>;
    async fn ensure_active_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord>;
    async fn refresh_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord>;
}

#[async_trait::async_trait]
impl<R> TokenManager<R> for TokenManagerS<R>
where
    R: TokenRepository,
{
    fn new(repo: R) -> Self {
        Self {
            repo,
            tokens: HashMap::new(),
            providers: HashMap::new(),
        }
    }

    fn register_provider(&mut self, variant: ProviderVariant, provider: Arc<dyn TokenProvider>) {
        self.providers.insert(variant, provider);
    }

    fn get_provider(&self, variant: ProviderVariant) -> Option<Arc<dyn TokenProvider>> {
        self.providers.get(&variant).cloned()
    }

    fn token(&self, provider: ProviderVariant, account: AccountVariant) -> Option<&TokenRecord> {
        self.tokens.get(&(provider, account))
    }

    async fn get_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord> {
        if let Some(token) = self.token(provider, account) {
            return Ok(*token.clone());
        }

        let token = self.repo.get_provider_token(provider, account).await?;

        let provider_instance = self.providers.get(&provider);
        if let Some(provider_instance) = provider_instance {
            let token = provider_instance.get_token(account).await?;
        }
        self.tokens.insert((provider, account), token.clone());
        Ok(token);
        todo!()
    }

    async fn ensure_active_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord> {
        let provider_instance = self.get_provider(provider);
        if let Some(provider_instance) = provider_instance {}
    }

    async fn refresh_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord> {
        todo!()
    }
}
