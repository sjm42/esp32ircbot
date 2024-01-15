// apiserver.rs

use axum::{
    extract::Path, http::StatusCode, response::Html, routing::get, routing::post, Json, Router,
};
use log::*;
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
// use tower_http::trace::TraceLayer;

use crate::*;

#[derive(Debug, Deserialize)]
pub struct Say {
    text: String,
}

pub async fn api_server(state: Arc<RwLock<MyState>>) -> anyhow::Result<()> {
    let listen = format!("0.0.0.0:{}", env!("API_PORT"));
    let addr = listen.parse::<SocketAddr>()?;

    let app = Router::new()
        .route(
            "/",
            get({
                let index = "<HTML></HTML>".to_string();
                move || async { Html(index) }
            }),
        )
        .route(
            "/cmd/:op",
            get({
                // let state = Arc::clone(&shared_state);
                let state = state.clone();
                move |path| cmd(path, state)
            }),
        )
        .route(
            "/say",
            post({
                let state = state.clone();
                move |body| say(body, state)
            }),
        );
    // .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening to {listen}");
    Ok(axum::serve(listener, app.into_make_service()).await?)
}

async fn cmd(Path(op): Path<String>, state: Arc<RwLock<MyState>>) -> (StatusCode, String) {
    let status;
    {
        let mut st = state.write().await;
        st.cnt += 1;
        status = format!("Cmd# {c} op: {op}", c = st.cnt);
    }
    info!("Status: {status}");
    (StatusCode::OK, status)
}

async fn say(Json(say): Json<Say>, state: Arc<RwLock<MyState>>) -> (StatusCode, String) {
    let status;
    {
        let mut st = state.write().await;
        st.cnt += 1;
        status = format!("Cmd# {c} say: {say:?}\n", c = st.cnt);
        if let Some(s) = &st.snd {
            s.send_privmsg(&st.bcfg.as_ref().unwrap().channel, say.text)
                .ok();
        }
    }
    info!("Status: {status}");
    (StatusCode::OK, status)
}

// EOF
