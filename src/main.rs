mod handler;
mod files;
mod filectx;
mod reactor;
mod timer;
mod signals;
mod framer;
mod pipes;

use std::{sync::{Arc, Mutex, atomic::Ordering}, time::Duration};

use nix::sys::signal::Signal::SIGUSR1;
use reactor::Reactor;

use crate::{pipes::PipeOperations, files::FileOperation, signals::SignalOperations, timer::TimerOperation};

fn main() {

    let mut rct = Reactor::new();

    let _res = rct.read_named_pipe("thepipe", |ctx| {

        ctx.on_chunk(|data, _ctx|{
            let s = String::from_utf8(data).unwrap();
            println!("{s}");
        });
        
    });

    // let _ = rct.read_file("Cargo.toml", |ctx|{
    //     // ctx.on_chunk(|data, ctx| ctx.push_bytes(&data) );
    //     ctx.on_eof(|ctx| {
    //         let v = ctx.take();
    //         let s = String::from_utf8_lossy(&v);
    //         println!("{s}");
    //     });
    //
    // });

    // let s = String::from("Hello, this is a test\n");
    //
    // let _ = rct.write_file("text.txt", Some(20), s.into_bytes(), |_data, n| {
    //     if n < 20 {
    //         println!("file written");
    //     }
    // });

    // let _ = rct.start_timer(Duration::from_secs(3), || println!("Timer expired"));

    rct.run();

}
