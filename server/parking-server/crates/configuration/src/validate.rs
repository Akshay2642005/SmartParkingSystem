pub(crate) fn validate_configuration(config: &crate::schema::Config) -> anyhow::Result<()> {
    ensure_not_blank(&config.primary.env, "primary.env")?;
    ensure_not_blank(&config.primary.name, "primary.name")?;
    Ok(())
}
fn ensure_not_blank(value: &str, field: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} cannot be blank");
    }
    Ok(())
}
