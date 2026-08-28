#![allow(unused)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PER_PAGE: u64 = 20;
const MAX_PER_PAGE: u64 = 100;

#[derive(Debug, Clone, Default)]
pub struct CreateFields {
    pub metadata: Option<serde_json::Value>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateFields {
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE,
            per_page: DEFAULT_PER_PAGE,
        }
    }
}

impl Page {
    #[inline]
    pub fn page(&self) -> u64 {
        self.page.max(1)
    }
    #[inline]
    pub fn per_page(&self) -> u64 {
        self.per_page.clamp(1, MAX_PER_PAGE)
    }

    #[inline]
    pub fn zero_indexed(&self) -> u64 {
        self.page().saturating_sub(1)
    }
}

#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, total: u64, p: &Page) -> Self {
        let per_page = p.per_page();
        let total_pages = total.div_ceil(per_page).max(1);
        Self {
            items,
            total,
            page: p.page(),
            per_page,
            total_pages,
        }
    }
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}
fn default_per_page() -> u64 {
    DEFAULT_PER_PAGE
}
