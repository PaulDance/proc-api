use anyhow::Result;

mod proc;

fn main() -> Result<()> {
    dbg!(proc::collect_procs()?);
    Ok(())
}
