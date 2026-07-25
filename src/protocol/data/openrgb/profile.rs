use serde::{Deserialize, Serialize};

use crate::impl_bufserde_json;

/// Data in Profile JSON
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ProfileData {
    base_color: u32,
    profile_name: String,
    profile_version: usize,
    controllers: serde_json::Value,
}

impl_bufserde_json!(ProfileData);
