#[derive(Debug, Clone)]
pub enum TokenEnum {
    Twitch {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<i64>,
        user_id: String,
    },
}

impl serde::Serialize for TokenEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        match self {
            TokenEnum::Twitch {
                access_token,
                refresh_token,
                expires_at,
                user_id,
            } => {
                let mut state = serializer.serialize_struct("TokenEnum", 5)?;
                state.serialize_field("type", "twitch")?;
                state.serialize_field("access_token", access_token)?;
                if let Some(rt) = refresh_token {
                    state.serialize_field("refresh_token", rt)?;
                }
                if let Some(exp) = expires_at {
                    state.serialize_field("expires_at", exp)?;
                }
                state.serialize_field("user_id", user_id)?;
                state.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for TokenEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::Deserialize;
        use serde::de::Error;
        #[derive(Deserialize)]
        struct TwitchTokenRaw {
            #[serde(rename = "type")]
            _type: String,
            access_token: String,
            refresh_token: Option<String>,
            expires_at: Option<i64>,
            user_id: String,
        }
        let raw = TwitchTokenRaw::deserialize(deserializer)?;
        Ok(TokenEnum::Twitch {
            access_token: raw.access_token,
            refresh_token: raw.refresh_token,
            expires_at: raw.expires_at,
            user_id: raw.user_id,
        })
    }
}
