use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct VersionInfo {
    pub version: &'static str,
    pub revision: &'static str,
}

pub async fn get_version() -> Json<VersionInfo> {
    Json(VersionInfo {
        version: env!("CARGO_PKG_VERSION"),
        revision: env!("VERGEN_GIT_SHA"),
    })
}
