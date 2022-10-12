use axum::{
    body::Bytes, extract::Query, http::StatusCode, response::IntoResponse, routing::post, Router,
};
use serde::Deserialize;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/master", post(modify_master))
        .route("/media", post(modify_media));
    let addr = env::var("LISTEN_ADDRESS").expect("env var LISTEN_ADDRESS is not set");
    let socket_addr = addr
        .parse::<SocketAddr>()
        .expect("value for LISTEN_ADDRESS must be like 127.0.0.1:3000");
    tracing::debug!("listening on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn modify_master(params: Query<Params>, body: Bytes) -> impl IntoResponse {
    match manifest_filter::load_master(&body) {
        Ok(pl) => {
            let mut master = manifest_filter::Master { playlist: pl };
            master
                .filter_bandwidth(params.min_bitrate, params.max_bitrate)
                .filter_fps(params.rate)
                .first_variant_by_index(params.variant_index)
                .first_variant_by_closest_bandwidth(params.closest_bandwidth);

            let mut v: Vec<u8> = Vec::new();
            master.playlist.write_to(&mut v).unwrap();

            (StatusCode::OK, String::from_utf8(v).unwrap())
        }
        Err(e) => (StatusCode::BAD_REQUEST, e),
    }
}

async fn modify_media(params: Query<Params>, body: Bytes) -> impl IntoResponse {
    match manifest_filter::load_media(&body) {
        Ok(pl) => {
            let mut media = manifest_filter::Media { playlist: pl };
            media
                .filter_dvr(params.dvr)
                .trim(params.trim_start, params.trim_end);

            let mut v: Vec<u8> = Vec::new();
            media.playlist.write_to(&mut v).unwrap();

            (StatusCode::OK, String::from_utf8(v).unwrap())
        }
        Err(e) => (StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize, Default)]
struct Params {
    min_bitrate: Option<u64>,
    max_bitrate: Option<u64>,
    rate: Option<f64>,
    dvr: Option<u64>,
    trim_start: Option<u64>,
    trim_end: Option<u64>,
    variant_index: Option<u64>,
    closest_bandwidth: Option<u64>,
}
