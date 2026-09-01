use configuration::Config;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};

use crate::{
    domain::parking::parse_topic, events::ServerEvent, protocol::TopicKind, state::AppState,
};
const MQTT_CLIENT_ID: &str = "parking-server";
const MQTT_TOPIC: &str = "parking/#";

pub async fn run(state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (client, mut event_loop) = create_client(&state.config)?;

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

                handle_publish(&state, &publish.topic, &publish.payload).await;
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

async fn handle_publish(state: &AppState, topic: &str, payload: &[u8]) {
    match parse_topic(topic) {
        Ok((_, _, TopicKind::Status)) => match state.store.apply_status(topic, payload).await {
            // Liveness handling (dashboard node_status events) comes later;
            // today offline matters because it resets the stale-seq baseline.
            Ok(outcome) => {
                info!(topic, ?outcome, "node status update");
                state.publish(ServerEvent::node_status(
                    outcome.site,
                    outcome.section,
                    outcome.status,
                ));
            }

            Err(error) => {
                warn!(topic, error = %error, "rejected MQTT message");
            }
        },

        _ => match state.store.apply_update(topic, payload).await {
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

pub fn spawn(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        info!("starting MQTT subscriber");

        if let Err(error) = run(state).await {
            tracing::error!(%error,"MQTT subscriber terminated");
        }
    })
}
