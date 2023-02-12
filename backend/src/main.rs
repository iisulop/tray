use std::net::SocketAddr;

use axum::response::IntoResponse;
use axum::{Router, Json};
use axum::routing::post;
use axum::http::StatusCode;
use http::{HeaderValue, Method};
use http::header::InvalidHeaderValue;
use serde::Deserialize;
use thiserror::Error;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Error, Debug)]
enum BeError {
    #[error("Cannot parse origin for `allow-origin`")]
    OriginParseError(#[from] InvalidHeaderValue)
}

#[tokio::main]
async fn main() -> Result<(), BeError> {
    println!("Hello, world!");
    // initialize tracing
    tracing_subscriber::fmt::init();
    let cors = CorsLayer::new()
        .allow_methods([Method::OPTIONS, Method::GET, Method::POST])
        .allow_origin("http://localhost:8000".parse::<HeaderValue>()?);

    // build our application with a route
    let app = Router::new()
        .layer(cors)
        // `POST /users` goes to `create_user`
        .route("/vote/", post(vote));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Deserialize)]
struct CastVote {
    number: usize,
}

async fn vote(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(vote): Json<CastVote>,
) -> impl IntoResponse {
    info!("Voting {}", vote.number);
    println!("Voting {}", vote.number);
    (StatusCode::OK, ())
}

