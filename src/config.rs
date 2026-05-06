use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const CONFIG_FILE: &str = "config.json";
pub const DEFAULT_EXPIRATION_DAYS: u32 = 30;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub expiration_enabled: bool,
    #[serde(default = "default_expiration_days")]
    pub expiration_days: u32,
    #[serde(default)]
    pub admin_password_hash: String,
    #[serde(default)]
    pub admin_salt: String,
    #[serde(default = "default_chat_enabled")]
    pub chat_enabled: bool,
}

fn default_expiration_days() -> u32 {
    DEFAULT_EXPIRATION_DAYS
}

fn default_chat_enabled() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            expiration_enabled: false,
            expiration_days: DEFAULT_EXPIRATION_DAYS,
            admin_password_hash: String::new(),
            admin_salt: String::new(),
            chat_enabled: true,
        }
    }
}

fn config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE)
}

/// Read config.json from CWD; return defaults if missing or unparseable.
pub fn load() -> Config {
    match std::fs::read_to_string(config_path()) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// Persist `cfg` to CWD/config.json (atomic via tmp+rename).
pub fn save(cfg: &Config) -> std::io::Result<()> {
    let data = serde_json::to_string_pretty(cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let tmp = config_path().with_extension("json.tmp");
    std::fs::write(&tmp, &data)?;
    std::fs::rename(&tmp, config_path())?;
    Ok(())
}

impl Config {
    pub fn is_admin_configured(&self) -> bool {
        !self.admin_password_hash.is_empty() && !self.admin_salt.is_empty()
    }

    pub fn verify_password(&self, password: &str) -> bool {
        if !self.is_admin_configured() {
            return false;
        }
        let Ok(salt_bytes) = hex::decode(&self.admin_salt) else {
            return false;
        };
        let mut hasher = Sha256::new();
        hasher.update(&salt_bytes);
        hasher.update(password.as_bytes());
        let computed = hex::encode(hasher.finalize());
        // Constant-time-ish comparison; the inputs are fixed length hex so
        // a normal eq is acceptable here, but compare bytes to avoid
        // short-circuiting on the first mismatched character.
        let a = computed.as_bytes();
        let b = self.admin_password_hash.as_bytes();
        if a.len() != b.len() {
            return false;
        }
        let mut diff: u8 = 0;
        for i in 0..a.len() {
            diff |= a[i] ^ b[i];
        }
        diff == 0
    }
}
