use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub type Db = Arc<Mutex<Vec<Todo>>>;

pub fn init_db() -> Db {
    Arc::new(Mutex::new(Vec::new()))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: usize,
    pub content: String,
    pub complete: bool,
}

#[derive(Debug, Deserialize)]
pub struct QueryOptions {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub complete: Option<bool>,
}
