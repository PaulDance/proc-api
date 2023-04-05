use anyhow::Result;

mod proc;
use proc::ProcInfo;

fn main() -> Result<()> {
    dbg!(ProcInfo::collect_all()?);
    Ok(())
}
