use axum::response::IntoResponse;
use entity::{candidate, poll, vote};
use http::{header::InvalidHeaderValue, StatusCode};
use sea_orm::{DbConn, FromQueryResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub type Id = i32;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct PollPost {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "camelCase")]
pub struct PollResponse {
    pub id: Id,
    pub title: String,
    pub creation_time: String,
    pub candidate_ids: Vec<Id>,
}

impl From<poll::Model> for PollResponse {
    fn from(src: poll::Model) -> Self {
        Self {
            id: src.id,
            title: src.title,
            creation_time: src.creation_time,
            candidate_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidatePost {
    pub url: Url,
    pub poll_id: Id,
}

#[derive(FromQueryResult)]
#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateResponse {
    pub id: Id,
    pub url: String,
    pub poll_id: Id,
    pub num_votes: i32,
}

impl From<candidate::Model> for CandidateResponse {
    fn from(src: candidate::Model) -> Self {
        Self {
            id: src.id,
            url: src.url,
            poll_id: src.poll_id,
            num_votes: 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "camelCase")]
pub struct VoteResponse {
    pub id: Id,
    pub candidate_id: Id,
    pub creation_time: String,
}

impl From<vote::Model> for VoteResponse {
    fn from(src: vote::Model) -> Self {
        Self {
            id: src.id,
            candidate_id: src.candidate_id,
            creation_time: src.creation_time,
        }
    }
}

