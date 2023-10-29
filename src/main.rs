mod config;
mod db;
mod features;
mod state;

use std::sync::Arc;

use axum::{Router, Server};

use config::Config;
use state::AppState;
use tower_http::trace::TraceLayer;

fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .nest("/user", features::users::router(state.clone()))
        .nest("/auth", features::auth::router(state.clone()))
}

#[tokio::main]
async fn main() {
    match dotenvy::dotenv() {
        Err(e) => eprintln!("Warning: .env file failed to load: {}", e),
        _ => {}
    };

    let config = Config::from_env();
    let db = db::db_connect(&config.db_url).await;
    let state = Arc::new(AppState::new(db, config));

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = router(state.clone())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .into_make_service();

    let addr = (([0, 0, 0, 0], (8000))).into();
    Server::bind(&addr).serve(app).await.unwrap();
}
