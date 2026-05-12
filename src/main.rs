mod handler;
mod files;
mod reactor;
mod timer;


use std::{sync::{Arc, Mutex}, time::Duration};

use reactor::Reactor;

use crate::{files::FileOperation, timer::TimerOperation};

fn main() {

    let mut rct = Reactor::new();

    let all_data = Arc::new(Mutex::new(Vec::<u8>::new()));
    let all_data_chunk = all_data.clone();

    let _ = rct.read_file("Cargo.toml", Some(20), move |data, n| {

        all_data_chunk.lock().unwrap().extend_from_slice(&data[..n]);

    });

    let s = String::from("Hello, this is a test\n");

    let _ = rct.write_file("text.txt", Some(20), s.into_bytes(), |_data, n| {
        if n < 20 {
            println!("file written");
        }
    });

    let _ = rct.start_timer(Duration::from_secs(3), || println!("Timer expired"));

    rct.run();

    let s = String::from_utf8(all_data.lock().unwrap().to_vec()).unwrap();
    println!("{s}");
}
