
mod reactor;
use reactor::Reactor;

fn main() {

    let mut rct = Reactor::new();

    rct.read_file("./example.txt", |content:String| println!("{content}") );

    rct.run();
}
