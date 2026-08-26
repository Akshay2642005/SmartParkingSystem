use crate::{schema::Config, validate::validate_configuration};
use config::{Config as C, ConfigBuilder, File, FileFormat};
use std::path::Path;

fn add_base_config(
    builder: ConfigBuilder<config::builder::DefaultState>,
    config_dir: &Path,
) -> ConfigBuilder<config::builder::DefaultState> {
    let yaml = config_dir.join("config.yml");
    let toml = config_dir.join("config.toml");

    if yaml.exists() {
        builder.add_source(File::from(yaml).format(FileFormat::Yaml).required(true))
    } else if toml.exists() {
        builder.add_source(File::from(toml).format(FileFormat::Toml).required(true))
    } else {
        panic!("No Default config file found (expected config.yml or config.toml)");
    }
}

pub fn load_config(config_dir: &Path) -> anyhow::Result<Config> {
    let _ = dotenvy::dotenv();
    let builder = C::builder();
    let builder = add_base_config(builder, config_dir);
    let config = builder.build()?.try_deserialize::<Config>()?;
    validate_configuration(&config)?;
    Ok(config)
}
