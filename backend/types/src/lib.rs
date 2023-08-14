use entity::{poll, candidate, vote};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use url::Url;

pub type Id = i32;

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

