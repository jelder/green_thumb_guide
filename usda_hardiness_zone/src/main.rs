use axum::response::IntoResponse;
use axum::response::Response;
use axum::{
    routing::get,
    Json,
    Router,
    extract::{Query, Extension},
    debug_handler,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::path::Path;
use maud::{html, Markup};


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

struct HardinessZone {
    id: String,
    min_temp_f: f64,
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

impl From<HardinessZone> for ZipcodeLookupResult {
    fn from(hardiness_zone: HardinessZone) -> Self {
        ZipcodeLookupResult {
            zone: hardiness_zone.id.to_string(),
            min_temp_f: hardiness_zone.min_temp_f,
            min_temp_c: fahrenheit_to_celsius(hardiness_zone.min_temp_f),
        }
    }
}

#[debug_handler]
async fn lookup_by_zipcode(
    Query(params): Query<QueryParams>,
    Extension(pool): Extension<SqlitePool>,
) -> Response {
    
    let hardiness_zone = sqlx::query_as!(HardinessZone, "select zones.* from zones join zone_zipcodes on zone_id = zones.id where zipcode = $1", params.q)
        .fetch_optional(&pool)
        .await;

    match hardiness_zone {
        Ok(Some(zone)) => {
            let result = ZipcodeLookupResult::from(zone);
            Json(result).into_response()
        },
        Err(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
        },
        _ => {
            StatusCode::NOT_FOUND.into_response()
        }
    }

}

#[tokio::main]
async fn main() {
    eprintln!("Starting server...");

    // Connect to SQLite database
    let pool = SqlitePool::connect(
        &first_existing_path(&vec!["/opt/database/hardiness.db", "/opt/hardiness.db", "./hardiness.db"]).expect("Could not find database")
    ).await.unwrap();

    let app = Router::new()
        .route("/", get(lookup_by_zipcode))
        .route("/hardiness_zone", get(lookup_by_zipcode))
        .route("/privacy_policy", get(|| async { privacy_policy() }))
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


fn first_existing_path(paths: &[&str]) -> Option<String> {
    paths.iter()
         .find(|&&path| Path::new(path).exists())
         .map(|&path| path.to_string())
}

fn privacy_policy() -> Markup {
    html! {
        head {
            link rel="stylesheet" href="https://edwardtufte.github.io/tufte-css/tufte.css";
        }
        body {
            h1 { "Privacy Policy" }
            p { "This site does not collect any personal information." }
        }
    }
}