mod handler;
mod files;
mod reactor;

use reactor::Reactor;

fn main() {


    let mut rct = Reactor::new();

    let _ = rct.read_file("Cargo.toml", |data, n| {

        let s = String::from_utf8(data).unwrap();
        println!("{n} bytes red\n{s}");

    });

    let buffer:String = String::from("HEllo\n");

    let _ = rct.write_file("text.txt", Vec::from(buffer), |_data, n|{

        println!("wrote bytes: {n}");

    });

    rct.run();
}
