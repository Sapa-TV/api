use sqlx::prelude::Type;

#[derive(Type, Hash, PartialEq, Eq, Clone, Debug)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum ProviderVariant {
    Twitch,
}

#[derive(Type, Hash, PartialEq, Eq, Clone, Debug)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum AccountVariant {
    Main,
    Bot,
}

#[derive(Clone)]
pub struct TokenRecord {
    pub account_variant: AccountVariant,
    pub provider: ProviderVariant,
    pub token: crate::token_manager::domain::enums::TokenEnum,
}
