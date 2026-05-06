use lazy_static::lazy_static;
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_TTL_SECS: u64 = 8 * 60 * 60; // 8h

lazy_static! {
    static ref SESSIONS: Mutex<HashMap<String, u64>> = Mutex::new(HashMap::new());
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn create_token() -> String {
    let mut buf = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    let token = hex::encode(buf);
    if let Ok(mut sessions) = SESSIONS.lock() {
        // Opportunistic GC of expired tokens.
        let now = now_secs();
        sessions.retain(|_, exp| *exp > now);
        sessions.insert(token.clone(), now + SESSION_TTL_SECS);
    }
    token
}

pub fn validate(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let now = now_secs();
    let Ok(mut sessions) = SESSIONS.lock() else {
        return false;
    };
    match sessions.get(token).copied() {
        Some(exp) if exp > now => {
            // Sliding window — refresh expiry on every successful check.
            sessions.insert(token.to_string(), now + SESSION_TTL_SECS);
            true
        }
        Some(_) => {
            sessions.remove(token);
            false
        }
        None => false,
    }
}

pub fn revoke(token: &str) {
    if let Ok(mut sessions) = SESSIONS.lock() {
        sessions.remove(token);
    }
}

pub const COOKIE_NAME: &str = "shareboxx_admin";
pub const COOKIE_MAX_AGE: u64 = SESSION_TTL_SECS;
