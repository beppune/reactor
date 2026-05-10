
mod reactor;
use reactor::Reactor;

fn main() {

    let mut rct = Reactor::new();

    let _ = rct.read_file("Cargo.toml");

    rct.run();
}
