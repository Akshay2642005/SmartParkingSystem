use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub broker_uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub http_host: String,
    pub http_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let broker_uri =
            std::env::var("BROKER_URI").unwrap_or_else(|_| "mqtt://localhost:1883".to_string());

        // Validate the URI during startup instead of discovering an invalid
        // broker configuration after the subscriber task starts.
        let parsed = Url::parse(&broker_uri)?;

        if parsed.scheme() != "mqtt" {
            return Err(format!("unsupported broker URI scheme: {}", parsed.scheme()).into());
        }

        Ok(Self {
            broker_uri,
            username: std::env::var("MQTT_USERNAME").ok(),
            password: std::env::var("MQTT_PASSWORD").ok(),
            http_host: std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            http_port: std::env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
        })
    }
}
