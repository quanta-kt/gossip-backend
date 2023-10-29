use crate::{config::Config, db::Db};

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Config,
}

impl AppState {
    pub fn new(db: Db, config: Config) -> AppState {
        AppState { db, config }
    }
}
