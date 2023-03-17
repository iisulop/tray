use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Json, Path, State};
use axum::http::StatusCode;
use chrono::prelude::*;
use entity::{candidate, poll, vote};
use sea_orm::{ActiveModelTrait, DbConn, EntityTrait, Set};
use tracing::info;

use crate::types::{
    AppState, CandidatePost, CandidateResponse, Id, PollPost, PollResponse, VoteResponse,
};

pub async fn post_poll(
    State(state): State<AppState>,
    Json(poll): Json<PollPost>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let poll_row = poll::ActiveModel {
        title: Set(poll.title),
        creation_time: Set(Utc::now().to_rfc3339()),
        ..Default::default()
    };
    let poll_row: poll::Model = poll_row.insert(&state.conn).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save poll: {err}"),
        )
    })?;
    info!("Saved new poll {}: {}", poll_row.title, poll_row.id);
    println!();
    Ok((
        StatusCode::CREATED,
        serde_json::to_string(&PollResponse::from(poll_row)).unwrap(),
    ))
}

pub async fn get_poll(
    state: State<AppState>,
    Path(poll_id): Path<Id>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let poll_row = poll::Entity::find_by_id(poll_id)
        .one(&state.conn)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get poll {poll_id}: {err}"),
            )
        })?;
    match poll_row {
        Some(poll_row) => Ok((
            StatusCode::OK,
            serde_json::to_string(&PollResponse::from(poll_row)).unwrap(),
        )),
        None => Ok((StatusCode::NOT_FOUND, String::from(""))),
    }
}

pub async fn post_candidate(
    State(state): State<AppState>,
    Json(candidate): Json<CandidatePost>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let candidate_row = candidate::ActiveModel {
        url: Set(candidate.url.into()),
        poll_id: Set(candidate.poll_id),
        ..Default::default()
    };
    let candidate_row: candidate::Model =
        candidate_row.insert(&state.conn).await.map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save candidate: {err}"),
            )
        })?;
    info!(
        "Saved new candidate {} - {}: {}",
        candidate_row.poll_id, candidate_row.url, candidate_row.id
    );
    Ok((
        StatusCode::CREATED,
        serde_json::to_string(&CandidateResponse::from(candidate_row)).unwrap(),
    ))
}

async fn construct_candidate(
    conn: &DbConn,
    candidate_id: &Id,
    candidate_row: entity::candidate::Model,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let vote_count = candidate::Entity::find_by_id(*candidate_id)
        .find_with_related(vote::Entity)
        //.select_only()
        //.column(vote::Column::Id)
        .all(conn)
        .await
        .iter()
        .count();
    dbg!(vote_count);
    let mut candidate = CandidateResponse::from(candidate_row);
    candidate.num_votes = vote_count;
    let candidate: String = serde_json::to_string(&candidate).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize candidate {candidate_id}: {err}"),
        )
    })?;

    Ok((StatusCode::OK, candidate))
}

pub async fn get_candidate(
    state: State<AppState>,
    Path(candidate_id): Path<Id>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    dbg!("get candidate", &candidate_id);
    let candidate_row = candidate::Entity::find_by_id(candidate_id)
        .one(&state.conn)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get candidate {candidate_id}: {err}"),
            )
        })?;
    match candidate_row {
        Some(candidate_row) => construct_candidate(&state.conn, &candidate_id, candidate_row).await,
        None => Ok((StatusCode::NOT_FOUND, String::from(""))),
    }
}

pub async fn vote(
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    state: State<AppState>,
    Path(candidate_id): Path<Id>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    info!("Voting {}", candidate_id);
    let vote_row = vote::ActiveModel {
        candidate_id: Set(candidate_id),
        source_ip: Set(address.to_string()),
        creation_time: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };
    let vote_row: vote::Model = vote_row.insert(&state.conn).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save vote: {err}"),
        )
    })?;
    info!("Saved new vote {}: {}", vote_row.candidate_id, vote_row.id);
    dbg!(&vote_row);
    Ok((
        StatusCode::OK,
        serde_json::to_string(&VoteResponse::from(vote_row)).unwrap(),
    ))
}
