use crate::pipes::PipeOperations;

mod handler;
mod files;
mod filectx;
mod reactor;
mod timer;
mod signals;
mod framer;
mod pipes;

fn main() {

        let mut r = reactor::Reactor::new();

        let _ = r.read_named_pipe("thepipe", |ctx| {
            ctx.on_chunk(|data, _| println!("{}", String::from_utf8(data).unwrap()));
            ctx.on_close(|_|println!("close pipe"));
        });

        r.run();
}
