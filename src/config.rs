use std::{fmt, fs, ops::Deref, time::Duration};

use serde::{de, Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_telegram_bot_token")]
    pub telegram_bot_token: Secret<String>,
    pub storage_path: String,
    /// Default chat message lifetime.
    /// Please [note][1] that message can only be deleted if it was sent less than 48 hours ago.
    /// [1]: https://core.telegram.org/bots/api#deletemessage
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_message_lifetime")]
    pub message_lifetime: Duration,
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_deletion_period")]
    pub deletion_period: Duration,
}

/// Obtains Docker Secret or corresponding environment variable value.
fn deserialize_telegram_bot_token<'de, D>(deserializer: D) -> Result<Secret<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer)
        .transpose()
        .unwrap_or_else(|| {
            fs::read_to_string("/run/secrets/telegram_bot_token")
                .map(Secret)
                .map_err(de::Error::custom)
        })
}

fn default_message_lifetime() -> Duration {
    Duration::from_secs(42 * 60 * 60)
}

fn default_deletion_period() -> Duration {
    Duration::from_secs(5 * 60)
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Secret<T>(T);

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[secret]")
    }
}

impl<T> Deref for Secret<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
