use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast};

use crate::error::{AppError, AppResult};
use crate::token::enums::TokenEnum;
use crate::token::provider::TokenProvider;
use crate::token::repository::TokenRepository;
use crate::token::types::{AccountVariant, ProviderVariant, TokenRecord};

pub struct TokenManager {
    repo: Arc<dyn TokenRepository + Send + Sync>,
    tokens: Arc<RwLock<HashMap<(ProviderVariant, AccountVariant), TokenRecord>>>,
    providers: Arc<RwLock<HashMap<ProviderVariant, Arc<dyn TokenProvider>>>>,
    token_change_tx: broadcast::Sender<()>,
}

impl TokenManager {
    pub fn new(repo: Arc<dyn TokenRepository + Send + Sync>) -> Self {
        let (token_change_tx, _) = broadcast::channel(1);
        Self {
            repo,
            tokens: Arc::new(RwLock::new(HashMap::new())),
            providers: Arc::new(RwLock::new(HashMap::new())),
            token_change_tx,
        }
    }

    pub async fn register_provider(
        &self,
        variant: ProviderVariant,
        provider: Arc<dyn TokenProvider>,
    ) {
        let mut providers = self.providers.write().await;
        providers.insert(variant, provider);
    }

    pub async fn get_provider(&self, variant: ProviderVariant) -> Option<Arc<dyn TokenProvider>> {
        let providers = self.providers.read().await;
        providers.get(&variant).cloned()
    }

    pub async fn get_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord> {
        let cache_key = (provider.clone(), account.clone());
        {
            let tokens = self.tokens.read().await;
            if let Some(token) = tokens.get(&cache_key) {
                return Ok(token.clone());
            }
        }

        let token_enum = self
            .repo
            .get_provider_token(provider.clone(), account.clone())
            .await?;

        match token_enum {
            Some(token_enum) => {
                let token_record = TokenRecord {
                    account_variant: account,
                    provider,
                    token: token_enum,
                };
                let mut tokens = self.tokens.write().await;
                tokens.insert(cache_key, token_record.clone());
                Ok(token_record)
            }
            None => Err(AppError::Internal("Token not found".to_string())),
        }
    }

    pub async fn ensure_active_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord> {
        let token = self.get_token(provider.clone(), account.clone()).await?;

        let provider_instance = self.get_provider(provider.clone()).await;
        if let Some(provider_instance) = provider_instance {
            match provider_instance.validate_refresh_token(&token).await {
                Ok(new_token_enum) => {
                    let new_token_record = TokenRecord {
                        account_variant: account.clone(),
                        provider: provider.clone(),
                        token: new_token_enum.clone(),
                    };
                    let mut tokens = self.tokens.write().await;
                    tokens.insert(
                        (provider.clone(), account.clone()),
                        new_token_record.clone(),
                    );
                    return Ok(new_token_record);
                }
                Err(_) => {
                    tracing::debug!("Token validation failed, attempting refresh");
                    return self.refresh_token(provider, account).await;
                }
            }
        }

        Ok(token)
    }

    pub async fn refresh_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
    ) -> AppResult<TokenRecord> {
        let token = self.get_token(provider.clone(), account.clone()).await?;

        let provider_instance = self.get_provider(provider.clone()).await;
        if let Some(provider_instance) = provider_instance {
            let old_user_id = match &token.token {
                TokenEnum::Twitch { user_id, .. } => user_id.clone(),
            };

            tracing::info!(
                "Refreshing token: provider={:?}, account={:?}, user_id={}",
                provider,
                account,
                old_user_id
            );

            let new_token_enum = provider_instance.force_refresh_token(&token).await?;
            let new_user_id = match &new_token_enum {
                TokenEnum::Twitch { user_id, .. } => user_id.clone(),
            };
            let new_expires_at = match &new_token_enum {
                TokenEnum::Twitch { expires_at, .. } => expires_at,
            };

            tracing::info!(
                "Token refreshed: user_id={}, new_expires_at={:?}",
                new_user_id,
                new_expires_at
            );

            let new_token_record = TokenRecord {
                account_variant: account.clone(),
                provider: provider.clone(),
                token: new_token_enum.clone(),
            };

            self.repo
                .save_provider_token(
                    provider.clone(),
                    account.clone(),
                    "",
                    chrono::Utc::now(),
                    new_token_enum,
                )
                .await?;

            let mut tokens = self.tokens.write().await;
            tokens.insert((provider, account), new_token_record.clone());
            let _ = self.token_change_tx.send(());
            return Ok(new_token_record);
        }

        Err(AppError::Internal("Provider not found".to_string()))
    }

    pub async fn exchange_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
        code: &str,
    ) -> AppResult<TokenEnum> {
        let provider_instance = self
            .get_provider(provider.clone())
            .await
            .ok_or_else(|| AppError::Internal("Provider not found".to_string()))?;

        tracing::debug!(
            "Exchanging code for token: provider={:?}, account={:?}",
            provider,
            account
        );

        let token_enum = provider_instance.exchange_token(code).await?;

        let user_id = match &token_enum {
            TokenEnum::Twitch { user_id, .. } => user_id.clone(),
        };

        tracing::info!(
            "Token exchanged, saving to repository: provider={:?}, account={:?}, user_id={}",
            provider,
            account,
            user_id
        );

        let token_record = TokenRecord {
            account_variant: account.clone(),
            provider: provider.clone(),
            token: token_enum.clone(),
        };

        self.repo
            .save_provider_token(
                provider.clone(),
                account.clone(),
                "",
                chrono::Utc::now(),
                token_enum.clone(),
            )
            .await?;

        let mut tokens = self.tokens.write().await;
        tokens.insert((provider.clone(), account.clone()), token_record);
        let _ = self.token_change_tx.send(());

        tracing::info!("Token saved to cache and repository");

        Ok(token_enum)
    }

    pub async fn generate_url(&self, provider: ProviderVariant) -> AppResult<String> {
        let provider_instance = self
            .get_provider(provider.clone())
            .await
            .ok_or_else(|| AppError::Internal("Provider not found".to_string()))?;

        provider_instance.generate_url()
    }

    pub fn subscribe_token_changes(&self) -> broadcast::Receiver<()> {
        self.token_change_tx.subscribe()
    }
}
