use crate::{domain::parking::SectionState, response::error::AppError, store::SharedStore};

fn storage_error(error: impl std::fmt::Display) -> AppError {
    tracing::error!(%error, "parking state read failed");
    AppError::service_unavailable("parking state is temporarily unavaliable")
}

pub async fn list_all(store: &SharedStore) -> Result<Vec<SectionState>, AppError> {
    let mut sections = store.snapshot_all().await.map_err(storage_error)?;

    sections.sort_by(|left, right| (&left.site, &left.section).cmp(&(&right.site, &right.section)));
    Ok(sections)
}

pub async fn list_site(store: &SharedStore, site: &str) -> Result<Vec<SectionState>, AppError> {
    let sections = list_all(store)
        .await?
        .into_iter()
        .filter(|section| section.site == site)
        .collect::<Vec<_>>();

    if sections.is_empty() {
        return Err(AppError::not_found(format!(
            "no sections reporting for site {site:?}"
        )));
    }
    Ok(sections)
}

pub async fn get(store: &SharedStore, site: &str, section: &str) -> Result<SectionState, AppError> {
    store
        .get(site, section)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            AppError::not_found(format!("no state for section {section:?} of site {site:?}"))
        })
}
