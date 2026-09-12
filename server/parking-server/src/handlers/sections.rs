use axum::{
    Json,
    extract::{Path, State},
};
use services::sections as svc;

use crate::{AppState, domain::parking::SectionState, response::error::AppError, services};

pub async fn list_sections(
    State(state): State<AppState>,
) -> Result<Json<Vec<SectionState>>, AppError> {
    Ok(Json(svc::list_all(&state.store).await?))
}

pub async fn list_site_sections(
    State(state): State<AppState>,
    Path(site): Path<String>,
) -> Result<Json<Vec<SectionState>>, AppError> {
    Ok(Json(svc::list_site(&state.store, &site).await?))
}

pub async fn get_section(
    State(state): State<AppState>,
    Path((site, section)): Path<(String, String)>,
) -> Result<Json<SectionState>, AppError> {
    Ok(Json(svc::get(&state.store, &site, &section).await?))
}
