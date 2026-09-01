use crate::{state::AppState, store};
use anyhow::{Context, Result};
use axum::Router;
use configuration::Config;
use seaorm::SeaOrmStore;
use std::{future::Future, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{info, warn};

pub struct Server {
    listener: TcpListener,
    app: Router,
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
        let Server { listener, app } = self;
        let addr = listener
            .local_addr()
            .context("could not read local address")?;
        info!(%addr, "http server started");

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown)
        .await
        .context("server error")
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
        let app = build_router(state, &self.cfg);

        let addr: SocketAddr =
            format!("{}:{}", self.cfg.server.host, self.cfg.server.port).parse()?;
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind TCP listener to {addr}"))?;
        info!(%addr, "tcp listener bound — ready to serve");

        Ok(Server { listener, app })
    }
}

pub fn build_router(_state: AppState, _cfg: &Config) -> Router {
    Router::new()
}
