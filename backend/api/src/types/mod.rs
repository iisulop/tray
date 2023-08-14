use axum::response::IntoResponse;
use entity::{candidate, poll, vote};
use http::{header::InvalidHeaderValue, StatusCode};
use sea_orm::{DbConn, FromQueryResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum BeError {
    #[error("Cannot parse origin for `allow-origin`")]
    OriginParseError(#[from] InvalidHeaderValue),
    #[error("Cannot connect to database")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Cannot serialize value")]
    SerializationError(#[from] serde_json::Error),
}

impl IntoResponse for BeError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub conn: DbConn,
}

impl AppState {
    pub fn with_conn(conn: DbConn) -> Self {
        Self { conn }
    }
}

