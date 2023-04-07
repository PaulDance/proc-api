use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::Serialize;
use sysinfo::{PidExt, ProcessExt, System, SystemExt, UserExt};
use tokio::sync::{broadcast, RwLock};

pub type ProcCache = Arc<RwLock<CacheInner>>;
type CacheData = HashSet<ProcInfo>;

#[derive(Debug)]
pub struct CacheInner {
    cache: CacheData,
    channel: broadcast::Sender<Vec<ProcInfo>>,
}

impl Default for CacheInner {
    fn default() -> Self {
        Self {
            cache: CacheData::default(),
            channel: broadcast::channel(Self::CHAN_CAP).0,
        }
    }
}

impl CacheInner {
    const CHAN_CAP: usize = 16;

    pub fn get(&self) -> &CacheData {
        &self.cache
    }

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

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<ProcInfo>> {
        self.channel.subscribe()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct ProcInfo {
    pub pid: u32,
    pub uid: u32,
    pub name: String,
    pub username: String,
}

impl ProcInfo {
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
