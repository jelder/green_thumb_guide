use axum::body::Body;
use axum::{
    routing::get,
    // Json,
    Router,
};
use hyper::Request;
// use hyper::StatusCode;
// use serde::{Deserialize, Serialize};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use sqlx::{Connection, SqliteConnection};

const DATABASE: &'static [u8] = include_bytes!("../hardiness.db");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();


    // Trace every request
    let trace_layer =
        TraceLayer::new_for_http().on_request(|_: &Request<Body>, _: &tracing::Span| {
            tracing::info!(message = "begin request")
        });

    let memory_database = SqliteConnection::connect(":memory:").await.unwrap();
    memory_database.execute(sqlx::query("ATTACH DATABASE ? AS embedded", DATABASE)).await.unwrap();

    async fn get_zone_by_zipcode() -> &'static str {
    }

    // Wrap an `axum::Router` with our state, CORS, Tracing, & Compression layers
    let app = Router::new()
        .route("/", get(get_zone_by_zipcode))
        .layer(trace_layer)
        .layer(CompressionLayer::new().gzip(true).deflate(true));

    #[cfg(debug_assertions)]
    {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    // If we compile in release mode, use the Lambda Runtime
    #[cfg(not(debug_assertions))]
    {
        // To run with AWS Lambda runtime, wrap in our `LambdaLayer`
        let app = tower::ServiceBuilder::new()
            .layer(axum_aws_lambda::LambdaLayer::default())
            .service(app);

        lambda_http::run(app).await.unwrap();
    }
}