use base64ct::{Base64UrlUnpadded, Encoding};
use sqlx::SqlitePool;
use web_push_native::{
    Auth, WebPushBuilder, jwt_simple::algorithms::ES256KeyPair, p256::PublicKey,
};

use crate::error::AppResult;
use crate::push::domain::PushSubscription;
use crate::push::domain::PushSubscriptionRepository;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PushSubscriptionRow {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub user_id: Option<String>,
}

impl From<PushSubscriptionRow> for PushSubscription {
    fn from(row: PushSubscriptionRow) -> Self {
        Self {
            endpoint: row.endpoint,
            p256dh: row.p256dh,
            auth: row.auth,
            user_id: row.user_id,
        }
    }
}

#[derive(Clone)]
pub struct SqlitePushSubscriptionRepository {
    db: SqlitePool,
}

impl SqlitePushSubscriptionRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl PushSubscriptionRepository for SqlitePushSubscriptionRepository {
    async fn insert_subscription(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO push_subscriptions (endpoint, p256dh, auth, user_id) VALUES (?, ?, ?, ?)",
        )
        .bind(endpoint)
        .bind(p256dh)
        .bind(auth)
        .bind(user_id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>> {
        let results = sqlx::query_as::<_, PushSubscriptionRow>(
            "SELECT endpoint, p256dh, auth, user_id FROM push_subscriptions",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(results.into_iter().map(|r| r.into()).collect::<Vec<_>>())
    }

    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
            .bind(endpoint)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

pub struct PushClient {
    client: reqwest::Client,
    key_pair: ES256KeyPair,
    contact: String,
}

impl PushClient {
    pub fn from_env() -> Option<Self> {
        let private_key = std::env::var("VAPID_PRIVATE_KEY").ok()?;
        let contact = std::env::var("VAPID_CONTACT").ok()?;
        Self::new(&private_key, &contact)
    }

    pub fn new(private_key: &str, contact: &str) -> Option<Self> {
        let vapid_key = Base64UrlUnpadded::decode_vec(private_key).ok()?;
        let key_pair = ES256KeyPair::from_bytes(&vapid_key).ok()?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        tracing::info!("PushClient initialized with contact: {}", contact);

        Some(Self {
            client,
            key_pair,
            contact: contact.to_string(),
        })
    }

    pub async fn send_to_all(
        &self,
        subscriptions: &[PushSubscription],
        title: &str,
        body: &str,
    ) -> u32 {
        tracing::info!(
            "Sending push to {} subscribers: title='{}', body='{}'",
            subscriptions.len(),
            title,
            body
        );

        let payload = format!(
            r#"{{"title":"{}","body":"{}","icon":"/icon.png"}}"#,
            title, body
        );
        let mut sent = 0u32;

        for sub in subscriptions {
            let endpoint = match sub.endpoint.parse() {
                Ok(e) => e,
                Err(_) => {
                    tracing::warn!("Failed to parse endpoint for subscription");
                    continue;
                }
            };

            let p256dh = match Base64UrlUnpadded::decode_vec(&sub.p256dh) {
                Ok(k) => k,
                Err(_) => {
                    tracing::warn!("Failed to decode p256dh for subscription");
                    continue;
                }
            };

            let auth = match Base64UrlUnpadded::decode_vec(&sub.auth) {
                Ok(a) => a,
                Err(_) => {
                    tracing::warn!("Failed to decode auth for subscription");
                    continue;
                }
            };

            let public_key = match PublicKey::from_sec1_bytes(&p256dh) {
                Ok(k) => k,
                Err(_) => {
                    tracing::warn!("Failed to create public key for subscription");
                    continue;
                }
            };

            let builder = WebPushBuilder::new(endpoint, public_key, Auth::clone_from_slice(&auth));
            let builder = builder.with_vapid(&self.key_pair, &self.contact);

            let request = builder.build(payload.as_bytes().to_vec());

            if let Ok(req) = request {
                let uri = req.uri().to_string();
                let method = req.method().clone();
                let headers = req.headers().clone();
                let body = req.into_body();

                match self
                    .client
                    .request(method, &uri)
                    .headers(headers)
                    .body(body)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            tracing::info!(
                                "Push sent successfully to user_id={:?}, endpoint status={}",
                                sub.user_id,
                                response.status()
                            );
                            sent += 1;
                        } else {
                            tracing::warn!(
                                "Push failed to user_id={:?}, endpoint status={}",
                                sub.user_id,
                                response.status()
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Push request error for user_id={:?}: {}", sub.user_id, e);
                    }
                }
            } else {
                tracing::warn!("Failed to build push request for user_id={:?}", sub.user_id);
            }
        }

        tracing::info!(
            "Push completed: {}/{} sent successfully",
            sent,
            subscriptions.len()
        );

        sent
    }
}
