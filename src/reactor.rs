use std::collections::HashMap;

use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};
use std::sync::mpsc;
use std::thread;

use nix::libc::{POLLIN, POLLOUT, TX_ANNOUNCE};
use nix::poll::{PollFd, PollFlags, PollTimeout};

use crate::handler::{Handler, Action, Interest};

pub struct Reactor {
    fds: HashMap<RawFd, (OwnedFd, Box<dyn Handler>, Interest)>,
}

impl Reactor {

    pub fn new() -> Self {
        Self {
            fds: HashMap::new(),
        }
    }

    pub fn register(&mut self, ofd:OwnedFd, handler: Box<dyn Handler>, int:Interest) {
        self.fds.insert(ofd.as_raw_fd(), (ofd, handler, int));
    }

    pub fn run(&mut self) {
        let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>();

        let executor_thread = thread::spawn(move ||{
            loop {
                match rx.recv() {
                    Ok(task) => task(),
                    Err(_) => break ,
                }
            }
        });

        loop {
            if self.fds.is_empty() {
                break;
            }

            // poll
            let mut topolls:Vec<RawFd> = vec![];
            {
                let mut p:Vec<PollFd> = self.fds.values().map(|(ofd, _, _)| PollFd::new( ofd.as_fd(), PollFlags::from_bits(POLLIN | POLLOUT).unwrap()) ).collect();
                let n = nix::poll::poll(&mut p, PollTimeout::NONE).unwrap();
                if n > 0 {
                    topolls = p.iter()
                        .filter(|p| p.revents().is_some_and(|f| !f.is_empty()))
                        .map(|k|k.as_fd().as_raw_fd())
                        .collect();
                }
            }
            // demux
            for fd in topolls {
                let (ofd, hnd, _int) = self.fds.get_mut(&fd).unwrap();
                let action = hnd.handle(ofd.as_fd());

                match action {
                    Action::Stop => { 
                        self.fds.remove(&fd).unwrap();
                    },
                    Action::Continue => {

                    },
                    Action::Task(task) => {
                        self.fds.remove(&fd).unwrap();
                        let _ = tx.send(task);
                    }
                }

            }
            // demux
        }
        // loop
        drop(tx);
        let _ = executor_thread.join();
    }


}
