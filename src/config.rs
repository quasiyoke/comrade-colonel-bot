use std::{fmt, fs, ops::Deref, time::Duration};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_telegram_bot_token")]
    pub telegram_bot_token: Secret<String>,
    pub storage_path: String,
    /// Please [note][1] that message can only be deleted if it was sent less than 48 hours ago.
    /// [1]: https://core.telegram.org/bots/api#deletemessage
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_message_lifetime")]
    pub message_lifetime: Duration,
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_deletion_period")]
    pub deletion_period: Duration,
    #[serde(default = "default_nodelete_hashtags")]
    pub nodelete_hashtags: Vec<String>,
}

/// Obtains Docker Secret when environment variable isn't specified.
fn default_telegram_bot_token() -> Secret<String> {
    fs::read_to_string("/run/secrets/telegram_bot_token")
        .map(Secret)
        .expect("please specify telegram_bot_token as a Docker secret or environment variable")
}

fn default_message_lifetime() -> Duration {
    Duration::from_secs(42 * 60 * 60)
}

fn default_deletion_period() -> Duration {
    Duration::from_secs(5 * 60)
}

fn default_nodelete_hashtags() -> Vec<String> {
    vec!["nodelete".into()]
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
