use chrono::{DateTime, Local, TimeDelta};

pub(crate) struct StorageValue {
    value: String,
    created_at: DateTime<Local>,
    key_settings: KeySettings,
}

impl StorageValue {
    pub(crate) fn new(value: String, key_settings: KeySettings) -> Self {
        Self {
            value,
            created_at: Local::now(),
            key_settings,
        }
    }

    pub(crate) fn is_value_expired(&self) -> bool {
        if self.key_settings.expiry.is_none() {
            return false;
        }

        let expired_at = self.created_at + TimeDelta::try_milliseconds(500).unwrap();
        expired_at >= Local::now()
    }
}

impl AsRef<str> for StorageValue {
    fn as_ref(&self) -> &str {
        &self.value
    }
}


pub struct KeySettings {
    expiry: Option<u64>,
}

pub struct KeySettingsBuilder {
    settings: KeySettings,
}

impl KeySettingsBuilder {
    pub fn new() -> Self {
        Self {
            settings: KeySettings { expiry: None },
        }
    }

    pub fn with_expiry(mut self, expiry: u64) -> Self {
        self.settings.expiry = Some(expiry);
        self
    }

    pub fn build(self) -> KeySettings {
        self.settings
    }
}

