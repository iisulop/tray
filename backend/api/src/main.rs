use std::env;
use std::net::SocketAddr;

use tracing::debug;

use api::types::{AppState, BeError};
use api::{db_connect, router};

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
    debug!("listening on {}", addr);
    println!("Started server on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();

    Ok(())
}
