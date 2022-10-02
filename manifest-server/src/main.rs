use axum::{
    body::Bytes, extract::Query, http::StatusCode, response::IntoResponse, routing::post, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", post(modify_manifest));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn modify_manifest(params: Query<Params>, body: Bytes) -> impl IntoResponse {
    match manifest_filter::load_master(&body) {
        Ok(pl) => {
            let mpl = manifest_filter::filter_bandwidth(
                pl,
                manifest_filter::BandwidthFilter {
                    min: params.min_bitrate,
                    max: params.max_bitrate,
                },
            );
            let mut v: Vec<u8> = Vec::new();
            mpl.write_to(&mut v).unwrap();

            (StatusCode::OK, String::from_utf8(v).unwrap())
        }
        Err(e) => (StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize, Default)]
struct Params {
    min_bitrate: Option<u64>,
    max_bitrate: Option<u64>,
}
