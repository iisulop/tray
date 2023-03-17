use std::env;
use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use chrono::prelude::*;
use http::header::InvalidHeaderValue;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, Database, DbConn, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower_http::cors::CorsLayer;
use tracing::info;

use entity::{candidate, poll, vote};
use url::Url;

type Id = i32;

#[derive(Error, Debug)]
enum BeError {
    #[error("Cannot parse origin for `allow-origin`")]
    OriginParseError(#[from] InvalidHeaderValue),
    #[error("Cannot connect to database")]
    DatabaseError(#[from] sea_orm::DbErr),
}

#[derive(Clone, Debug)]
struct AppState {
    conn: DbConn,
}

impl AppState {
    fn with_conn(conn: DbConn) -> Self {
        Self { conn }
    }
}

async fn db_migrate(db_conn: &DbConn) -> Result<(), BeError> {
    info!("Running database migrations");
    Ok(Migrator::up(db_conn, None).await?)
}

async fn db_connect(db_url: &str) -> Result<DbConn, BeError> {
    let db_conn = Database::connect(db_url).await?;
    db_migrate(&db_conn).await?;
    Ok(db_conn)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/poll", post(post_poll))
        .route("/poll/:poll_id", get(get_poll))
        .route("/candidate", get(get_candidate).post(post_candidate))
        .route("/candidate/:candidate_id/vote", get(vote))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), BeError> {
    tracing_subscriber::fmt::init();

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_err| String::from("sqlite::memory:"));
    let db_conn = db_connect(&db_url).await?;

    let state = AppState::with_conn(db_conn);
    let app = router(state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    tracing::debug!("listening on {}", addr);
    println!("Started server on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename = "camelCase")]
struct VoteResponse {
    id: Id,
    candidate_id: Id,
    creation_time: String,
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

async fn vote(
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
    Ok((
        StatusCode::CREATED,
        serde_json::to_string(&VoteResponse::from(vote_row)).unwrap(),
    ))
}

#[derive(Debug, Deserialize, Serialize)]
struct PollPost {
    title: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "camelCase")]
struct PollResponse {
    id: Id,
    title: String,
    creation_time: String,
}

impl From<poll::Model> for PollResponse {
    fn from(src: poll::Model) -> Self {
        Self {
            id: src.id,
            title: src.title,
            creation_time: src.creation_time,
        }
    }
}

async fn post_poll(
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

async fn get_poll(
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

#[derive(Debug, Deserialize, Serialize)]
struct CandidatePost {
    url: Url,
    poll_id: Id,
}

#[derive(Debug, Deserialize, Serialize)]
struct CandidateResponse {
    id: Id,
    url: String,
    poll_id: Id,
}

impl From<candidate::Model> for CandidateResponse {
    fn from(src: candidate::Model) -> Self {
        Self {
            id: src.id,
            url: src.url,
            poll_id: src.poll_id,
        }
    }
}

async fn post_candidate(
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

async fn get_candidate(
    state: State<AppState>,
    Path(id): Path<Id>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let candidate_row = candidate::Entity::find_by_id(id)
        .one(&state.conn)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get candidate {id}: {err}"),
            )
        })?;
    match candidate_row {
        Some(candidate_row) => Ok((
            StatusCode::OK,
            serde_json::to_string(&CandidateResponse::from(candidate_row)).unwrap(),
        )),
        None => Ok((StatusCode::NOT_FOUND, String::from(""))),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        net::{SocketAddr, TcpListener},
        str::from_utf8,
        sync::Mutex,
    };

    use axum::{
        body::Body,
        http::Request,
    };

    use http::StatusCode;

    use crate::{
        db_connect, router, AppState, CandidatePost, CandidateResponse, PollPost, PollResponse,
    };

    static INITIALIZED: Mutex<bool> = Mutex::new(false);

    fn init_subscriber() {
        let mut init = INITIALIZED.lock().unwrap();
        if !*init {
            *init = true;
            tracing_subscriber::fmt::init();
        }
    }

    async fn serve() -> SocketAddr {
        init_subscriber();
        let db_url =
            env::var("DATABASE_URL").unwrap_or_else(|_err| String::from("sqlite::memory:"));
        let db_conn = db_connect(&db_url).await.unwrap();
        let state = AppState::with_conn(db_conn);
        let router = router(state);
        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .unwrap()
        });
        println!("Started server on {addr}");
        addr
    }

    #[tokio::test]
    async fn test_post_poll() {
        let addr = serve().await;
        let poll = PollPost {
            title: String::from("Test title #1"),
        };

        let client = hyper::Client::new();
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .uri(format!("http://{addr}/poll"))
                    .body(Body::from(serde_json::to_string(&poll).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        dbg!(&response);
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_get_candidate() {
        let addr = serve().await;
        let poll = create_poll(&addr, "Test poll #4").await;

        let client = hyper::Client::new();
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::GET)
                    .uri(format!("http://{addr}/poll/{}", poll.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        dbg!(&response);
        assert_eq!(response.status(), StatusCode::OK);
    }

    /*
    async fn response_to<'a, T, Y>(response: Response<Y>) -> T
        where
            T: serde::Deserialize<'a>,
            Y: axum::body::HttpBody + std::fmt::Debug, <Y as HttpBody>::Error: std::fmt::Debug,
    {
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        serde_json::from_str::<T>(
            from_utf8(
                &body[..]
            ).unwrap()
        ).unwrap()
    }
    */

    async fn create_poll(addr: &SocketAddr, poll_name: &str) -> PollResponse {
        let poll = PollPost {
            title: String::from(poll_name),
        };

        let client = hyper::Client::new();
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .uri(format!("http://{addr}/poll"))
                    .body(Body::from(serde_json::to_string(&poll).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        // response_to::<PollResponse, _>(response);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        serde_json::from_str::<PollResponse>(from_utf8(&body[..]).unwrap()).unwrap()

        /*
        PollResponse {
            id: todo!(),
            title: todo!(),
            creation_time: todo!(),
        }
        */
    }

    #[tokio::test]
    async fn test_post_candidate() {
        let addr = serve().await;
        let poll = create_poll(&addr, "Test poll #2").await;

        let candidate = CandidatePost {
            url: "https://localhost:8080/im1.png".try_into().unwrap(),
            poll_id: poll.id,
        };

        let client = hyper::Client::new();
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .uri(format!("http://{addr}/candidate"))
                    .body(Body::from(serde_json::to_string(&candidate).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        dbg!(&response);
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    async fn create_candidate(addr: &SocketAddr, candidate: CandidatePost) -> CandidateResponse {
        let client = hyper::Client::new();
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .uri(format!("http://{addr}/candidate"))
                    .body(Body::from(serde_json::to_string(&candidate).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        serde_json::from_str::<CandidateResponse>(from_utf8(&body[..]).unwrap()).unwrap()
    }

    #[tokio::test]
    async fn test_vote() {
        let addr = serve().await;
        let poll = create_poll(&addr, "Test poll #3").await;

        let candidate = CandidatePost {
            url: "https://localhost:8080/im1.png".try_into().unwrap(),
            poll_id: poll.id,
        };
        let candidate = create_candidate(&addr, candidate).await;

        println!("Voting...");
        dbg!(&candidate);
        let client = hyper::client::Client::new();
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::GET)
                    .uri(format!("http://{addr}/candidate/{}/vote", candidate.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = from_utf8(&body[..]);
        dbg!(&body);

        //dbg!(&response);
        //assert_eq!(response.status(), StatusCode::CREATED);
    }
}
