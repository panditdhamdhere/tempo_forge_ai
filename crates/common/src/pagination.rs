use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PageParams {
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_limit")]
    pub limit: u32,

    #[serde(default)]
    pub cursor: Option<String>,

    #[serde(default)]
    pub q: Option<String>,
}

fn default_limit() -> u32 {
    20
}

impl Default for PageParams {
    fn default() -> Self {
        Self {
            limit: default_limit(),
            cursor: None,
            q: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}
