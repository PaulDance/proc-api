//! This module defines the fundamentals of the API: how to collect current
//! processes and how to store them in a common cache.

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::Serialize;
use sysinfo::{PidExt, ProcessExt, System, SystemExt, UserExt};
use tokio::sync::{broadcast, RwLock};

/// A read-write lock-synchronized cache for processes.
///
/// Instantiate using [`Default`].
pub type ProcCache = Arc<RwLock<CacheInner>>;
type CacheData = HashSet<ProcInfo>;

/// The effective storage for the [`ProcCache`]: a [`ProcInfo`] set and a
/// [`broadcast::channel`] as a means to support the streaming SSE endpoint.
///
/// The channel is an mpmc in order to only use it as an spmc. Messages are
/// vectors of [`ProcInfo`]s so that the need for synchronization can be
/// avoided as much as possible and so that the channel's capacity can be
/// bounded to the actual number of concurrent communications between the
/// refresh and stream endpoint handlers.
#[derive(Debug)]
pub struct CacheInner {
    cache: CacheData,
    channel: broadcast::Sender<Vec<ProcInfo>>,
}

/// Instantiates the cache with an empty storage and a channel with an arbitrary
/// but constant capacity.
impl Default for CacheInner {
    fn default() -> Self {
        Self {
            cache: CacheData::default(),
            channel: broadcast::channel(Self::CHAN_CAP).0,
        }
    }
}

impl CacheInner {
    /// Arbitrary default capacity for the channel backing the streaming.
    const CHAN_CAP: usize = 16;

    /// Returns the currently-cached process data.
    pub fn get(&self) -> &CacheData {
        &self.cache
    }

    /// Refresh the cache by collecting all processes currently running on the
    /// host again.
    ///
    /// The cache storage is entirely overwritten with the new data. If the
    /// channel has no receiver, then only that is done, otherwise new processes
    /// will be sent to all currently-subscribed receivers of the channel by
    /// computing the set difference between the new cache and the old one, but
    /// the cache is still completely overwritten in the end.
    pub fn refresh(&mut self) -> Result<()> {
        // Use the receiver count as an indicator of the current mode of
        // operation: 0 means blocking, anything else means streaming.
        if self.channel.receiver_count() == 0 {
            self.cache = ProcInfo::collect_all()?;
        } else {
            let old = self.cache.clone();
            self.cache = ProcInfo::collect_all()?;
            self.channel
                .send(self.cache.difference(&old).cloned().collect())?;
        }
        Ok(())
    }

    /// Generates a new receiver by subscribing to the backing channel.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<ProcInfo>> {
        self.channel.subscribe()
    }
}

/// Common information representing a process as handled via the API.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct ProcInfo {
    pub pid: u32,
    pub uid: u32,
    pub name: String,
    pub username: String,
}

impl ProcInfo {
    /// Collect all processes currently running on the host, blocking by nature.
    pub fn collect_all() -> Result<CacheData> {
        let mut res = CacheData::new();
        let sys = System::new_all();

        for (pid, proc) in sys.processes() {
            let uid = proc
                .user_id()
                .ok_or_else(|| anyhow!("Process {pid} does not have an associated user."))?;
            res.insert(Self {
                pid: pid.as_u32(),
                uid: **uid,
                username: sys
                    .get_user_by_id(uid)
                    .ok_or_else(|| anyhow!("Unable to retrieve process {pid}'s user name."))?
                    .name()
                    .to_owned(),
                name: proc.name().to_owned(),
            });
        }

        Ok(res)
    }
}
