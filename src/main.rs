mod handler;
mod files;
mod reactor;
mod timer;
mod signals;

use std::{sync::{Arc, Mutex}, time::Duration};

use nix::sys::signal::Signal::SIGUSR1;
use reactor::Reactor;

use crate::{files::FileOperation, signals::SignalOperations, timer::TimerOperation};

fn main() {

    let mut rct = Reactor::new();

    let all_data = Arc::new(Mutex::new(Vec::<u8>::new()));
    let all_data_chunk = all_data.clone();

    let _ = rct.read_file("Cargo.toml", Some(20), move |data, n| {

        all_data_chunk.lock().unwrap().extend_from_slice(&data[..n]);
        if n < 20 {
            let s = String::from_utf8(all_data.lock().unwrap().to_vec()).unwrap();
            println!("{s}");
        }

    });

    let s = String::from("Hello, this is a test\n");

    let _ = rct.write_file("text.txt", Some(20), s.into_bytes(), |_data, n| {
        if n < 20 {
            println!("file written");
        }
    });

    let _ = rct.start_timer(Duration::from_secs(3), || println!("Timer expired"));

    let _ = rct.on_signal( &[SIGUSR1], move |sig| {
        println!("A signal was caught: {sig}");
    });

    rct.run();

}
