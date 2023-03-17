use axum::routing::{get, post};
use axum::Router;
use sea_orm::{Database, DbConn};
use tower_http::cors::CorsLayer;
use tracing::info;

use migration::{Migrator, MigratorTrait};

pub mod controller;
pub mod types;

use controller::{get_candidate, get_poll, post_candidate, post_poll, vote};
use types::{AppState, BeError};

pub async fn db_migrate(db_conn: &DbConn) -> Result<(), BeError> {
    info!("Running database migrations");
    Ok(Migrator::up(db_conn, None).await?)
}

pub async fn db_connect(db_url: &str) -> Result<DbConn, BeError> {
    let db_conn = Database::connect(db_url).await?;
    db_migrate(&db_conn).await?;
    Ok(db_conn)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/poll", post(post_poll))
        .route("/poll/:poll_id", get(get_poll))
        .route("/candidate", post(post_candidate))
        .route("/candidate/:candidate_id", get(get_candidate))
        .route("/candidate/:candidate_id/vote", get(vote))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
