//! Server lifecycle and router composition.
//!
//! Route table:
//!
//! | Route | Purpose | Rate limited |
//! | ----- | ------- | ------------ |
//! | `GET /health`, `/healthz`, `/livez` | liveness | no |
//! | `GET /readyz` | readiness: device feed + store | no |
//! | `GET /status` | component detail | no |

use crate::{middleware, mqtt, state::AppState, store};
use anyhow::{Context, Result};
use axum::{Router, routing::get};
use configuration::Config;
use seaorm::SeaOrmStore;
use std::{future::Future, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task::JoinHandle};
use tracing::{info, warn};

pub struct Server {
    listener: TcpListener,
    app: Router,
    db: Option<SeaOrmStore>,
    ingest: JoinHandle<()>,
}

impl Server {
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.listener
            .local_addr()
            .context("could not read local address")
    }

    pub async fn run<S>(self, shutdown: S) -> Result<()>
    where
        S: Future<Output = ()> + Send + 'static,
    {
        let Server {
            listener,
            app,
            db,
            ingest,
        } = self;
        let addr = listener
            .local_addr()
            .context("could not read local address")?;
        info!(%addr, "http server started");

        let result = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown)
        .await
        .context("server error");

        ingest.abort();
        if let Some(db) = db
            && let Err(error) = db.close().await
        {
            tracing::warn!(%error, "failed to close database connections");
        }
        result
    }
}

pub struct ServerBuilder {
    cfg: Arc<Config>,
}

impl ServerBuilder {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self { cfg }
    }

    pub async fn build(self) -> Result<Server> {
        let db = match &self.cfg.store {
            Some(store) => {
                let db = SeaOrmStore::connect(store)
                    .await
                    .context("failed to connect to database")?;
                db.ping().await.context("database ping failed")?;
                Some(db)
            }
            None => {
                warn!(
                    "no store configured - running ephemeral: parking state and ingest guards \
                     are lost on restart"
                );
                None
            }
        };

        let parking_store: store::SharedStore = match db.clone() {
            None => Arc::new(store::MemoryStore::new()),
            Some(_db) => todo!(),
        };
        info!(
            backend = parking_store.backend(),
            "parking state backend ready"
        );

        let state = AppState::new(
            Arc::clone(&self.cfg),
            parking_store,
            crate::events::channel(),
        );

        let ingest = mqtt::spawn(state.clone());
        let app = build_router(state, &self.cfg);

        let addr: SocketAddr =
            format!("{}:{}", self.cfg.server.host, self.cfg.server.port).parse()?;
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind TCP listener to {addr}"))?;
        info!(%addr, "tcp listener bound — ready to serve");

        Ok(Server {
            listener,
            app,
            db,
            ingest,
        })
    }
}

/// Compose the router. Public for integration tests, which drive it directly.
pub fn build_router(state: AppState, cfg: &Config) -> Router {
    let router = ops_router().with_state(state);
    middleware::apply(router, cfg)
}

/// Unprefixed operational endpoints: liveness, readiness, component status.
fn ops_router() -> Router<AppState> {
    use crate::handlers::system;

    Router::new()
        .route("/health", get(system::health))
        .route("/healthz", get(system::healthz))
        .route("/livez", get(system::livez))
        .route("/readyz", get(system::readyz))
        .route("/status", get(system::status))
}
