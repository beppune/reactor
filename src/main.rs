mod handler;
mod files;
mod filectx;
mod reactor;
mod timer;
mod signals;
mod framer;
mod pipes;

use std::{io::{Write, stdin, stdout}, sync::{Arc, Mutex, atomic::Ordering}, time::Duration};

use nix::{libc::printf, sys::signal::Signal::SIGUSR1};
use reactor::Reactor;

use crate::{pipes::PipeOperations, files::FileOperation, signals::SignalOperations, timer::TimerOperation};

fn main() {

    let mut rct = Reactor::new();

    let s = String::from("ciaone\n");

    let _res = rct.write_named_pipe("thepipe", Vec::from(s), |ctx| {

        ctx.on_chunk(|_data, _ctx| println!("onchunk") );
        ctx.on_close(|_ctx| println!("close pipe") );

    });

    // let _res = rct.read_named_pipe("thepipe", |ctx| {
    //
    //     ctx.on_chunk(|data, _ctx|{
    //         println!("on_chunk");
    //         let s = String::from_utf8(data);
    //         match s {
    //             Ok(t) => println!("{t}"),
    //             Err(e) => println!("{:?}", e),
    //         }
    //     });
    //
    //     ctx.on_close(|_ctx| println!("closing pipe"));
    //
    // });

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
