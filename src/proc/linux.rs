use anyhow::{anyhow, Result};

use crate::proc::ProcInfo;

pub fn collect_procs() -> Result<Vec<ProcInfo>> {
    let mut res = Vec::new();

    for p in procfs::process::all_processes()? {
        let p = p?;
        let uid = p.uid()?;
        res.push(ProcInfo {
            pid: p.pid,
            uid,
            name: p.cmdline()?.first().unwrap_or(&String::new()).clone(),
            user: users::get_user_by_uid(uid)
                .ok_or_else(|| anyhow!("User {uid} not found."))?
                .name()
                .to_str()
                .ok_or_else(|| anyhow!("Invalid name."))?
                .to_string(),
        });
    }

    Ok(res)
}
