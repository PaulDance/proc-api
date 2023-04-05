use anyhow::{anyhow, Result};
use serde::Serialize;
use sysinfo::{PidExt, ProcessExt, System, SystemExt, UserExt};

#[derive(Debug, Serialize)]
pub struct ProcInfo {
    pub pid: u32,
    pub uid: u32,
    pub name: String,
    pub user: String,
}

impl ProcInfo {
    pub fn collect_all() -> Result<Vec<Self>> {
        let mut res = Vec::new();
        let sys = System::new_all();

        for (pid, proc) in sys.processes() {
            let uid = proc
                .user_id()
                .ok_or_else(|| anyhow!("Process {pid} does not have an associated user."))?;
            res.push(Self {
                pid: pid.as_u32(),
                uid: **uid,
                user: sys
                    .get_user_by_id(uid)
                    .ok_or_else(|| anyhow!("Unable to retrieve process {pid}'s user name."))?
                    .name()
                    .to_owned(),
                name: proc.name().to_owned(),
            })
        }

        Ok(res)
    }
}