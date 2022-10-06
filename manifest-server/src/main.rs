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
    let addr = env::var("LISTEN_ADDRESS").unwrap();
    let socket_addr = addr.parse::<SocketAddr>().unwrap();
    tracing::debug!("listening on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn modify_master(params: Query<Params>, body: Bytes) -> impl IntoResponse {
    match manifest_filter::load_master(&body) {
        Ok(pl) => {
            let mut mpl = manifest_filter::filter_bandwidth(
                pl,
                manifest_filter::BandwidthFilter {
                    min: params.min_bitrate,
                    max: params.max_bitrate,
                },
            );
            mpl = manifest_filter::filter_fps(mpl, params.rate);

            let mut v: Vec<u8> = Vec::new();
            mpl.write_to(&mut v).unwrap();

            (StatusCode::OK, String::from_utf8(v).unwrap())
        }
        Err(e) => (StatusCode::BAD_REQUEST, e),
    }
}

async fn modify_media(params: Query<Params>, body: Bytes) -> impl IntoResponse {
    match manifest_filter::load_media(&body) {
        Ok(pl) => {
            let mut mpl = manifest_filter::filter_dvr(pl, params.dvr);
            mpl = manifest_filter::trim(
                mpl,
                manifest_filter::TrimFilter {
                    start: params.trim_start,
                    end: params.trim_end,
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
    rate: Option<f64>,
    dvr: Option<u64>,
    trim_start: Option<u64>,
    trim_end: Option<u64>,
}
