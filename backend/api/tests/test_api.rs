use std::{
    env,
    net::{SocketAddr, TcpListener},
    str::from_utf8,
    sync::Mutex,
};

use axum::{body::Body, http::Request};

use http::StatusCode;

use api::{db_connect, router, types::Id};

use api::types::{AppState, CandidatePost, CandidateResponse, PollPost, PollResponse};

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
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_err| String::from("sqlite::memory:"));
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

async fn get_candidate(addr: &SocketAddr, candidate_id: Id) -> CandidateResponse {
    let client = hyper::Client::new();
    let response = client
        .request(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("http://{addr}/candidate/{}", candidate_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    dbg!(&body);
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
    let vote_response = client
        .request(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("http://{addr}/candidate/{}/vote", candidate.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    dbg!(&vote_response);
    assert_eq!(vote_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(vote_response.into_body())
        .await
        .unwrap();
    let body = from_utf8(&body[..]).unwrap();
    dbg!(body);

    let candidate_after = get_candidate(&addr, candidate.id).await;
    dbg!(&candidate_after);
    assert_eq!(candidate_after.num_votes, 1);
}
