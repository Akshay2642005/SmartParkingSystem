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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Serialized because the loader reads process-wide environment variables.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn write(dir: &Path, name: &str, body: &str) {
        let mut file = std::fs::File::create(dir.join(name)).unwrap();
        file.write_all(body.as_bytes()).unwrap();
    }

    #[test]
    fn boots_on_defaults_when_no_config_file_exists() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        let config = load_config(dir.path()).unwrap();

        assert_eq!(config.server.port, 8080);
        assert_eq!(config.mqtt.broker_uri, "mqtt://localhost:1883");
        assert!(config.store.is_none(), "defaults must stay ephemeral");
    }

    #[test]
    fn file_overrides_defaults_and_env_overrides_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        write(
            dir.path(),
            "config.yml",
            "server:\n  port: 9000\nmqtt:\n  client_id: from-file\n",
        );

        let from_file = load_config(dir.path()).unwrap();
        assert_eq!(from_file.server.port, 9000);
        assert_eq!(from_file.mqtt.client_id, "from-file");

        unsafe {
            std::env::set_var("APP_SERVER__PORT", "9999");
            std::env::set_var("APP_MQTT__CLIENT_ID", "from-env");
        }

        let from_env = load_config(dir.path()).unwrap();

        unsafe {
            std::env::remove_var("APP_SERVER__PORT");
            std::env::remove_var("APP_MQTT__CLIENT_ID");
        }

        assert_eq!(from_env.server.port, 9999, "env must win over the file");
        assert_eq!(from_env.mqtt.client_id, "from-env");
    }

    #[test]
    fn invalid_configuration_fails_the_load() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        write(dir.path(), "config.yml", "mqtt:\n  broker_uri: ftp://x\n");

        assert!(load_config(dir.path()).is_err());
    }

    #[test]
    fn unknown_keys_are_rejected_rather_than_silently_ignored() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        write(dir.path(), "config.yml", "server:\n  prot: 8080\n");

        assert!(load_config(dir.path()).is_err());
    }
}
