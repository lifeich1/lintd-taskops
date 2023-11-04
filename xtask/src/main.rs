use anyhow::Result;
use duct::cmd;
use lintd_taskops::ops::Recipe;
use lintd_taskops::Make;

struct Worker();
impl lintd_taskops::Addon for Worker {
    fn dist() -> Result<()> {
        println!("HIT users' dist receipt!");
        Ok(())
    }
    fn rule(target: String, _options: Vec<String>) -> Result<()> {
        match target.as_str() {
            "show-arch" => println!("uname: {}", cmd!("uname", "-a").eval()?),
            "help" => println!("all rule targets:\n  help, show-arch"),
            _ => (),
        }
        Ok(())
    }
}
fn main() -> Result<()> {
    let r = Worker::make();
    if let Err(e) = &r {
        println!("Detail: {e:?}");
    }
    r
}
