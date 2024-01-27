use axum::response::IntoResponse;
use axum::response::Response;
use axum::{
    debug_handler,
    extract::{Extension, Query},
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use maud::{html, Markup};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ZipcodeLookupQuery {
    q: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ZipcodeLookupResult {
    zone: String,
    min_temp_f: f64,
    min_temp_c: f64,
}

#[derive(Deserialize)]
struct QueryParams {
    q: String,
}

#[debug_handler]
async fn lookup_by_zipcode(
    Query(params): Query<QueryParams>,
    Extension(pool): Extension<SqlitePool>,
) -> Response {
    let result = sqlx::query!(
        "select zones.* from zones join zone_zipcodes on zone_id = zones.id where zipcode = $1",
        params.q
    )
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(hardiness_zone)) => Json(ZipcodeLookupResult {
            zone: hardiness_zone.id.to_string(),
            min_temp_f: hardiness_zone.min_temp_f,
            min_temp_c: fahrenheit_to_celsius(hardiness_zone.min_temp_f),
        })
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            simple_page("Internal Server Error", err.to_string().as_str()),
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            simple_page("Not Found", "The requested ZIP code could not be found."),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    eprintln!("Starting server...");

    // Connect to SQLite database
    let pool = SqlitePool::connect(
        &first_existing_path(&[
            "/opt/database/hardiness.db",
            "/opt/hardiness.db",
            "./hardiness.db",
        ])
        .expect("Could not find database"),
    )
    .await
    .unwrap();

    let app = Router::new()
        .route("/", get(lookup_by_zipcode))
        .route("/hardiness_zone", get(lookup_by_zipcode))
        .route(
            "/privacy_policy",
            get(|| async {
                simple_page(
                    "Privacy Policy",
                    "This site does not collect any personal information.",
                )
            }),
        )
        .layer(Extension(pool));

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

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    let c = (fahrenheit - 32.0) * 5.0 / 9.0;
    (c * 100.0).round() / 100.0
}

fn first_existing_path(paths: &[&str]) -> Option<String> {
    paths
        .iter()
        .find(|&&path| Path::new(path).exists())
        .map(|&path| path.to_string())
}

fn simple_page(title: &str, p: &str) -> Markup {
    html! {
        head { link rel="stylesheet" href="https://edwardtufte.github.io/tufte-css/tufte.css"; }
        body {
            h1 { (title) }
            p { (p) }
        }
    }
}
