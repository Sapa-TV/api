use twitch_api::twitch_oauth2::{Scope, Validator, validator};

macro_rules! define_scopes {
    ($($s:expr),* $(,)?) => {
        pub const TWITCH_SCOPES: &[Scope] = &[ $($s),* ];

        pub const TWITCH_SCOPES_VALIDATOR: Validator = validator!($($s),*);
    };
}

define_scopes![
    Scope::UserReadEmail,
    Scope::ChannelReadSubscriptions,
    Scope::ChannelReadGuestStar,
    Scope::UserReadChat,
    Scope::ChatEdit,
    Scope::ModerationRead,
    Scope::ChannelReadRedemptions,
    Scope::ChannelManageRedemptions,
    Scope::ChannelReadStreamKey,
];
