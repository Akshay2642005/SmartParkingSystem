use crate::state::{SharedStateStore, TopicKind, apply_status, apply_update, parse_topic};
use configuration::Config;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

const MQTT_CLIENT_ID: &str = "parking-server";
const MQTT_TOPIC: &str = "parking/#";

pub async fn run(
    config: std::sync::Arc<Config>,
    store: SharedStateStore,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (client, mut event_loop) = create_client(&config)?;

    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                info!("MQTT connected");

                client.subscribe(MQTT_TOPIC, QoS::AtLeastOnce).await?;

                info!("subscribed to {MQTT_TOPIC}");
            }

            Ok(Event::Incoming(Packet::Publish(publish))) => {
                info!(
                    topic = %publish.topic,
                    payload = %String::from_utf8_lossy(&publish.payload),
                    "MQTT publish received"
                );

                handle_publish(&publish.topic, &publish.payload, &store);
            }

            Ok(event) => {
                tracing::debug!(?event, "MQTT event");
            }

            Err(error) => {
                warn!("MQTT connection error: {error}");

                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn handle_publish(topic: &str, payload: &[u8], store: &SharedStateStore) {
    let mut guard = match store.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("state store mutex poisoned");
            poisoned.into_inner()
        }
    };

    match parse_topic(topic) {
        Ok((_, _, TopicKind::Status)) => match apply_status(&mut guard, topic, payload) {
            // Liveness handling (dashboard node_status events) comes later;
            // today offline matters because it resets the stale-seq baseline.
            Ok(status) => {
                info!(topic, ?status, "node status update");
            }

            Err(error) => {
                warn!(topic, error = %error, "rejected MQTT message");
            }
        },

        _ => match apply_update(&mut guard, topic, payload) {
            Ok(state) => {
                info!(
                    site = %state.site,
                    section = %state.section,
                    seq = state.seq,
                    slot_count = state.slot_count,
                    "accepted parking snapshot"
                );
            }

            Err(error) => {
                warn!(
                    topic = %topic,
                    error = %error,
                    "rejected MQTT message"
                );
            }
        },
    }
}

fn create_client(
    config: &Config,
) -> Result<(AsyncClient, EventLoop), Box<dyn std::error::Error + Send + Sync>> {
    let uri = url::Url::parse(&config.mqtt.broker_uri)?;

    let host = uri.host_str().ok_or("broker URI has no host")?;

    let port = uri.port().unwrap_or(1883);

    let mut options = MqttOptions::new(MQTT_CLIENT_ID, host, port);

    options.set_keep_alive(Duration::from_secs(30));

    if let Some(username) = &config.mqtt.username {
        options.set_credentials(
            username.expose(),
            config
                .mqtt
                .password
                .as_ref()
                .map(configuration::Secret::expose)
                .unwrap_or_default(),
        );
    }

    Ok(AsyncClient::new(options, 32))
}
