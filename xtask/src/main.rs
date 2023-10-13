use anyhow::Result;
fn main() -> Result<()> {
    let r = lintd_taskops::make();
    if let Err(e) = &r {
        println!("Detail: {e:?}");
    }
    r
}
