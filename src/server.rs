use std::{convert::Infallible, sync::Arc};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{
        Html, IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt, wrappers::WatchStream};

use crate::{
    anim,
    config::Config,
    state::AppState,
    theme::{self, Preset},
};

const WIDGET_HTML: &str = include_str!("../web/widget.html");
const CONFIG_HTML: &str = include_str!("../web/config.html");
const FONT: &[u8] = include_bytes!("../web/fonts/nunito.woff2");

type Shared = Arc<AppState>;

pub fn router(state: Shared) -> Router {
    Router::new()
        .route("/", get(|| async { Html(WIDGET_HTML) }))
        .route("/config", get(|| async { Html(CONFIG_HTML) }))
        .route("/fonts/nunito.woff2", get(font))
        .route("/events", get(events))
        .route("/cover", get(cover))
        .route("/animation", get(animation))
        .route("/api/config", get(get_config).post(post_config))
        .route(
            "/api/animation",
            axum::routing::post(upload_animation).layer(DefaultBodyLimit::max(anim::MAX_UPLOAD_BYTES)),
        )
        .layer(middleware::from_fn_with_state(state.clone(), local_only))
        .with_state(state)
}

pub async fn serve(listener: std::net::TcpListener, state: Shared) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::from_std(listener)?;
    axum::serve(listener, router(state)).await
}

/// Bloqueia DNS rebinding: só aceita requests endereçados ao próprio localhost.
async fn local_only(State(state): State<Shared>, request: Request, next: Next) -> Response {
    let host = request.headers().get(header::HOST).and_then(|h| h.to_str().ok());
    if host.is_some_and(|h| is_local_host(h, state.port)) {
        next.run(request).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

pub fn is_local_host(host: &str, port: u16) -> bool {
    host == format!("localhost:{port}") || host == format!("127.0.0.1:{port}")
}

async fn font() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "font/woff2"), (header::CACHE_CONTROL, "max-age=31536000")],
        FONT,
    )
}

async fn events(State(state): State<Shared>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let style = WatchStream::new(state.style.subscribe())
        .map(|style| Event::default().event("style").json_data(style));
    let now = WatchStream::new(state.now.subscribe())
        .map(|now| Event::default().event("now").json_data(now));
    let stream = style
        .merge(now)
        .filter_map(|event| event.ok())
        .map(Ok);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn cover(State(state): State<Shared>) -> Response {
    match state.cover.read().unwrap().as_ref() {
        Some(cover) => (
            [(header::CONTENT_TYPE, cover.content_type.clone()), (header::CACHE_CONTROL, "no-cache".into())],
            cover.bytes.clone(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn animation(State(state): State<Shared>) -> impl IntoResponse {
    let (bytes, mime) = state.animation();
    ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "no-cache")], bytes)
}

#[derive(Serialize)]
struct ConfigView {
    config: Config,
    presets: &'static [Preset],
    animations: Vec<String>,
}

fn config_view(state: &AppState, config: Config) -> Json<ConfigView> {
    Json(ConfigView {
        config,
        presets: theme::PRESETS,
        animations: anim::list(&state.assets_dir),
    })
}

async fn get_config(State(state): State<Shared>) -> Json<ConfigView> {
    config_view(&state, state.config())
}

async fn post_config(State(state): State<Shared>, Json(config): Json<Config>) -> Response {
    match state.apply_config(config) {
        Ok(config) => config_view(&state, config).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct UploadQuery {
    name: String,
}

async fn upload_animation(
    State(state): State<Shared>,
    Query(query): Query<UploadQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Exigir um Content-Type de imagem força preflight CORS: outros sites não conseguem enviar.
    let declared = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(anim::Format::from_mime);
    let Some(format) = anim::sniff(&body) else {
        return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported file").into_response();
    };
    if declared.is_none() {
        return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported content type").into_response();
    }
    match state.store_animation(&query.name, format, &body) {
        Ok(config) => config_view(&state, config).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    const PORT: u16 = 7878;

    struct TestDir(PathBuf);

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn setup(name: &str) -> (Shared, TestDir) {
        let dir = std::env::temp_dir().join(format!("pinguplay-server-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("assets")).unwrap();
        let state = Arc::new(AppState::new(
            Config::default(),
            dir.join("config.toml"),
            dir.join("assets"),
            PORT,
        ));
        (state, TestDir(dir))
    }

    fn request(method: &str, uri: &str) -> axum::http::request::Builder {
        Request::builder().method(method).uri(uri).header(header::HOST, format!("localhost:{PORT}"))
    }

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn local_host_check() {
        assert!(is_local_host("localhost:7878", 7878));
        assert!(is_local_host("127.0.0.1:7878", 7878));
        assert!(!is_local_host("evil.com:7878", 7878));
        assert!(!is_local_host("localhost:9999", 7878));
    }

    #[tokio::test]
    async fn rejects_foreign_host_header() {
        let (state, _dir) = setup("host");
        let response = router(state)
            .oneshot(Request::get("/").header(header::HOST, "evil.com:7878").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn serves_widget_and_default_animation() {
        let (state, _dir) = setup("pages");
        let app = router(state);
        let widget = app.clone().oneshot(request("GET", "/").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(widget.status(), StatusCode::OK);
        let anim = app.oneshot(request("GET", "/animation").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(anim.headers()[header::CONTENT_TYPE], "image/gif");
    }

    #[tokio::test]
    async fn posting_config_saves_sanitizes_and_publishes_style() {
        let (state, dir) = setup("config");
        let config = Config { theme: "mint".into(), width: 9999, ..Config::default() };
        let response = router(state.clone())
            .oneshot(
                request("POST", "/api/config")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&config).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["config"]["width"], 800);
        assert_eq!(json["presets"].as_array().unwrap().len(), theme::PRESETS.len());
        assert_eq!(state.style.borrow().bg, "#d4f5e6");
        assert_eq!(Config::load(&dir.0.join("config.toml")).theme, "mint");
    }

    #[test]
    fn language_change_notifies_listener() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let (state, _dir) = setup("lang");
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        state.on_lang_change(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        state.apply_config(Config { theme: "mint".into(), ..Config::default() }).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        state.apply_config(Config { lang: crate::config::Lang::En, ..Config::default() }).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn config_pointing_to_missing_animation_is_reset() {
        let (state, _dir) = setup("missing");
        let config = state.apply_config(Config { animation: "ghost.gif".into(), ..Config::default() }).unwrap();
        assert_eq!(config.animation, "");
    }

    #[tokio::test]
    async fn uploads_animation_and_bumps_revision() {
        let (state, dir) = setup("upload");
        let before = state.style.borrow().anim;
        let response = router(state.clone())
            .oneshot(
                request("POST", "/api/animation?name=Dance.gif")
                    .header(header::CONTENT_TYPE, "image/gif")
                    .body(Body::from(anim::DEFAULT_GIF))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["config"]["animation"], "Dance.gif");
        assert_eq!(json["animations"][0], "Dance.gif");
        assert!(dir.0.join("assets/Dance.gif").is_file());
        assert!(state.style.borrow().anim > before);
    }

    #[tokio::test]
    async fn rejects_non_image_uploads() {
        let (state, _dir) = setup("reject");
        let app = router(state);
        let not_image = app
            .clone()
            .oneshot(
                request("POST", "/api/animation?name=x.gif")
                    .header(header::CONTENT_TYPE, "image/gif")
                    .body(Body::from("MZ not a gif"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(not_image.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        let simple_request = app
            .oneshot(
                request("POST", "/api/animation?name=x.gif")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from(anim::DEFAULT_GIF))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(simple_request.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}
