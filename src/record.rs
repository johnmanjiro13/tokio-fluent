use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub tag: &'static str,
    pub timestamp: u64,
    pub entry: HashMap<String, String>,
}
