use crate::{models::*, services};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Clone, Default)]
pub struct Catalog(Arc<Mutex<CatalogState>>);

#[derive(Default)]
pub struct CatalogState {
    path: Option<PathBuf>,
    last: Option<Snapshot>,
    active_since: HashMap<String, u64>,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u64,
    destinations: Vec<Approval>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Approval {
    id: String,
    label: String,
    port: u16,
    version: String,
}

#[derive(Serialize)]
pub struct Destination {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Verified,
    Unreachable,
    IdentityUnverified,
    NotTestable,
}

#[derive(Serialize)]
pub struct TestResult {
    pub status: ConnectionStatus,
    pub revision: String,
}

impl Catalog {
    #[cfg(feature = "local-connection-fixture")]
    pub fn fixture(path: PathBuf) -> Self {
        let catalog = Self(Arc::new(Mutex::new(CatalogState {
            path: Some(path),
            ..CatalogState::default()
        })));
        let state = Arc::downgrade(&catalog.0);
        tokio::spawn(async move {
            let mut clock = tokio::time::interval(Duration::from_millis(150));
            loop {
                clock.tick().await;
                let Some(state) = state.upgrade() else { break };
                state.lock().await.refresh().await;
            }
        });
        catalog
    }

    pub async fn lock(&self) -> MutexGuard<'_, CatalogState> {
        self.0.lock().await
    }

    pub async fn destinations(&self) -> Items<Destination> {
        Items {
            items: self
                .lock()
                .await
                .refresh()
                .await
                .into_iter()
                .flat_map(|s| &s.destinations)
                .map(|a| Destination {
                    id: a.id.clone(),
                    label: a.label.clone(),
                })
                .collect(),
        }
    }

    pub async fn test(
        &self,
        pool: &SqlitePool,
        user: &User,
        agent: &str,
        revision: String,
    ) -> Result<TestResult, ApiError> {
        let status = tokio::time::timeout(
            Duration::from_secs(10),
            self.check(pool, user, agent, &revision),
        )
        .await
        .unwrap_or(Ok(ConnectionStatus::NotTestable))?;
        Ok(TestResult { status, revision })
    }

    #[cfg(feature = "local-connection-fixture")]
    async fn check(
        &self,
        pool: &SqlitePool,
        user: &User,
        agent: &str,
        revision: &str,
    ) -> Result<ConnectionStatus, ApiError> {
        use ConnectionStatus::*;
        use std::net::Ipv4Addr;
        use tokio::io::AsyncReadExt;

        let response_deadline = tokio::time::Instant::now() + Duration::from_secs(9);
        let mut catalog = self.lock().await;
        let Some(config) = services::connection_config(pool, user, agent)
            .await?
            .filter(|c| c.revision == revision)
        else {
            return Ok(NotTestable);
        };
        let Some(approval) = catalog.approval(&config.destination_id).await else {
            return Ok(NotTestable);
        };
        // Revocation and starting a socket contact share the same gate; never hold it for the identity response.
        let stream = tokio::time::timeout(
            Duration::from_secs(3),
            tokio::net::TcpStream::connect((Ipv4Addr::LOCALHOST, approval.0.port)),
        )
        .await;
        drop(catalog);

        let status = match stream {
            Ok(Ok(stream)) => {
                let mut payload = Vec::new();
                let read = tokio::time::timeout_at(response_deadline, async {
                    stream.take(1025).read_to_end(&mut payload).await
                })
                .await;
                if !matches!(read, Ok(Ok(_))) {
                    Unreachable
                } else {
                    let reply = (payload.len() <= 1024)
                        .then(|| serde_json::from_slice::<serde_json::Value>(&payload).ok())
                        .flatten();
                    match reply {
                        Some(value)
                            if value.as_str() == Some(config.expected_identity.as_str()) =>
                        {
                            Verified
                        }
                        Some(value) if value.get("redirect").is_some() => NotTestable,
                        _ => IdentityUnverified,
                    }
                }
            }
            _ => Unreachable,
        };
        let mut catalog = self.lock().await;
        let unchanged = services::connection_config(pool, user, agent)
            .await?
            .is_some_and(|c| c.revision == revision);
        if !unchanged || catalog.approval(&config.destination_id).await.as_ref() != Some(&approval)
        {
            return Ok(NotTestable);
        }
        Ok(status)
    }

    #[cfg(not(feature = "local-connection-fixture"))]
    async fn check(
        &self,
        pool: &SqlitePool,
        user: &User,
        agent: &str,
        _revision: &str,
    ) -> Result<ConnectionStatus, ApiError> {
        services::connection_config(pool, user, agent).await?;
        Ok(ConnectionStatus::NotTestable)
    }
}

impl CatalogState {
    pub async fn allows(&mut self, id: &str) -> bool {
        self.approval(id).await.is_some()
    }

    async fn approval(&mut self, id: &str) -> Option<(Approval, u64)> {
        let approval = self
            .refresh()
            .await?
            .destinations
            .iter()
            .find(|a| a.id == id)
            .cloned()?;
        Some((approval, *self.active_since.get(id)?))
    }

    async fn refresh(&mut self) -> Option<&Snapshot> {
        let path = self.path.clone()?;
        let next = tokio::time::timeout(
            Duration::from_secs(1),
            tokio::task::spawn_blocking(move || {
                use std::io::Read;
                if !std::fs::metadata(&path).ok()?.is_file() {
                    return None;
                }
                let mut bytes = Vec::new();
                std::fs::File::open(path)
                    .ok()?
                    .take(32_769)
                    .read_to_end(&mut bytes)
                    .ok()?;
                (bytes.len() <= 32_768)
                    .then(|| serde_json::from_slice::<Snapshot>(&bytes).ok())
                    .flatten()
            }),
        )
        .await
        .ok()?
        .ok()??;
        let mut ids = std::collections::HashSet::new();
        if next.version == 0
            || next.destinations.iter().any(|a| {
                a.id.trim().is_empty()
                    || a.label.trim().is_empty()
                    || a.port == 0
                    || a.version.trim().is_empty()
                    || !ids.insert(&a.id)
            })
        {
            return None;
        }
        if self
            .last
            .as_ref()
            .is_some_and(|last| next.version <= last.version)
        {
            return self.last.as_ref().filter(|last| next == **last);
        }
        // A restored entry starts a new approval even if the source reuses its old version.
        self.active_since = next
            .destinations
            .iter()
            .map(|entry| {
                let continuous = self
                    .last
                    .as_ref()
                    .is_some_and(|last| last.destinations.contains(entry));
                let epoch = if continuous {
                    self.active_since
                        .get(&entry.id)
                        .copied()
                        .unwrap_or(next.version)
                } else {
                    next.version
                };
                (entry.id.clone(), epoch)
            })
            .collect();
        eprintln!("Local test catalog activated: version {}", next.version);
        self.last = Some(next);
        self.last.as_ref()
    }
}
