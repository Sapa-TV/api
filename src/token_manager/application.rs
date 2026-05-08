use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast};

use crate::error::{AppError, AppResult};
use crate::token_manager::domain::enums::TokenEnum;
use crate::token_manager::domain::provider::TokenProvider;
use crate::token_manager::domain::repository::TokenRepository;
use crate::token_manager::domain::types::{AccountVariant, ProviderVariant, TokenRecord};

pub struct TokenManagerS {
    repo: Arc<dyn TokenRepository + Send + Sync>,
    tokens: Arc<RwLock<HashMap<(ProviderVariant, AccountVariant), TokenRecord>>>,
    providers: Arc<RwLock<HashMap<ProviderVariant, Arc<dyn TokenProvider>>>>,
    token_change_tx: broadcast::Sender<()>,
}

impl TokenManagerS {
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
                        token: new_token_enum,
                    };
                    let mut tokens = self.tokens.write().await;
                    tokens.insert(
                        (provider.clone(), account.clone()),
                        new_token_record.clone(),
                    );
                    return Ok(new_token_record);
                }
                Err(_) => {
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
            let new_token_enum = provider_instance.force_refresh_token(&token).await?;
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

        let token_enum = provider_instance.exchange_token(code).await?;

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
        tokens.insert((provider, account), token_record);
        let _ = self.token_change_tx.send(());

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
