use std::path::PathBuf;
use std::sync::Arc;

use super::proxy::{Method, Resolver, Server};
use super::EngineError;
use crate::catalog::{self, Region};

pub struct Engine {
    resolver: Arc<Resolver>,
}

pub struct Running {
    server: Server,
    active: bool,
}

impl Engine {
    pub fn new(_resources: PathBuf) -> Self {
        Self { resolver: Arc::new(Resolver::new()) }
    }

    pub fn profiles(region: Region) -> Vec<&'static str> {
        catalog::proxy_profiles(region)
    }

    pub fn recover() {
        #[cfg(target_os = "macos")]
        let _ = super::sysproxy::disable();
    }

    pub async fn prepare(&self) -> Result<(), EngineError> {
        Ok(())
    }

    pub async fn start(&self, profile: &str) -> Result<Running, EngineError> {
        let method = Method::parse(profile).ok_or_else(|| EngineError::Failed(format!("invalid profile: {profile}")))?;
        let server = Server::start(method, self.resolver.clone())
            .await
            .map_err(|e| EngineError::Failed(e.to_string()))?;
        Ok(Running { server, active: false })
    }
}

impl Running {
    pub fn alive(&mut self) -> bool {
        true
    }

    pub fn probe_proxy(&self) -> Option<u16> {
        Some(self.server.port())
    }

    pub async fn activate(&mut self) -> Result<(), EngineError> {
        #[cfg(target_os = "macos")]
        {
            let port = self.server.port();
            tokio::task::spawn_blocking(move || super::sysproxy::enable(port))
                .await
                .map_err(|e| EngineError::Failed(e.to_string()))??;
        }
        self.active = true;
        Ok(())
    }

    pub async fn stop(self) {
        #[cfg(target_os = "macos")]
        {
            if self.active {
                let _ = tokio::task::spawn_blocking(super::sysproxy::disable).await;
            }
        }
        drop(self.server);
    }
}
