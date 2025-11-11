//! Napoleonic weapon database

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDefinition {
    pub id: String,
    pub name: String,
}

pub struct WeaponDatabase {
    weapons: HashMap<String, WeaponDefinition>,
}

impl WeaponDatabase {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            weapons: HashMap::new(),
        })
    }
}
