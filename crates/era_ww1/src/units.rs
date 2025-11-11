//! WW1 unit definitions and database

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDefinition {
    pub id: String,
    pub name: String,
}

pub struct UnitDatabase {
    units: HashMap<String, UnitDefinition>,
}

impl UnitDatabase {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            units: HashMap::new(),
        })
    }
}
