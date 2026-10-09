//! Embedded SPA assets (05-04, D-01/D-02/D-03).
//!
//! The built `web/dist` tree is embedded into the `qai` binary at compile
//! time and served same-origin under every non-API path. `web/dist` is a
//! gitignored build artifact: with `allow_missing` a plain `cargo build`
//! succeeds before `npm run build` has run, and the fallback 404s until the
//! SPA is built (the `spa_fallback` contract test skips in that case).

use axum::{
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

/// Built SPA assets, embedded at compile time (`web/dist`, gitignored).
#[derive(Embed)]
#[folder = "../../web/dist/"]
#[allow_missing = true]
struct SpaAssets;

/// Whether the embed carries a built SPA (used for the environmental skip
/// in the fallback contract test).
pub fn spa_index_present() -> bool {
    SpaAssets::get("index.html").is_some()
}

/// Same-origin static fallback: serves the embedded asset for the exact
/// path, else `index.html` for client-side routes. Unknown `/api/*` paths
/// 404 here — the fallback never swallows the API (T-05-10). Only embedded
/// relative paths are resolved, so no traversal onto the filesystem (T-05-09).
pub async fn spa_fallback(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path == "api" || path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let (name, asset) = match SpaAssets::get(path) {
        Some(file) => (path, file),
        None => match SpaAssets::get("index.html") {
            Some(file) => ("index.html", file),
            None => {
                return (StatusCode::NOT_FOUND, "SPA not built: run `npm run build` in web/")
                    .into_response();
            }
        },
    };
    let mime = mime_guess::from_path(name).first_or_octet_stream();
    ([("content-type", mime.as_ref().to_string())], asset.data.to_vec()).into_response()
}
