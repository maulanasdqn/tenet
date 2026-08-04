#![allow(clippy::disallowed_methods)]

use std::env;

pub fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

pub fn env_parse<T: std::str::FromStr>(key: &str, default: &str) -> Result<T, String> {
    env_or(key, default)
        .parse()
        .map_err(|_| format!("{key} must be a number"))
}

pub fn env_flag_default(key: &str, default: &str) -> bool {
    matches!(env_or(key, default).as_str(), "1" | "true" | "yes")
}
