use core::task;
use std::collections::HashMap;

use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use nix::libc::{POLLIN, POLLOUT};
use nix::poll::{PollFd, PollFlags, PollTimeout};

use crate::handler::{Handler, Action, Interest};

#[derive(Clone)]
pub struct ReactorHandle {
    pub sender: Sender<Box<dyn FnOnce(&mut Reactor) + Send>>,
}

pub struct Reactor {
    fds: HashMap<RawFd, (OwnedFd, Box<dyn Handler>, Interest)>,
    pub command_sender: Sender<Box<dyn FnOnce(&mut Reactor) + Send>>,
    command_receiver: Receiver<Box<dyn FnOnce(&mut Reactor) + Send>>,
    
}

impl Reactor {

    pub fn new() -> Self {
        let (t, r) = mpsc::channel();
        Self {
            fds: HashMap::new(),
            command_sender: t,
            command_receiver: r,
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

            while let Ok(task) = self.command_receiver.try_recv() {
                (task)(self);
            }
            
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
            let mut toberemoved:Vec<i32> = vec![];
            for fd in topolls {
                let (ofd, hnd, _int) = self.fds.get_mut(&fd).unwrap();
                let action = hnd.handle(ofd.as_fd());

                match action {
                    Action::Stop => { 
                        toberemoved.push(fd);
                    },
                    Action::Continue => {

                    },
                    Action::Task(task) => {
                        let _ = tx.send(task);
                    },
                    Action::TaskAndStop(task) => {
                        let _ = tx.send(task);
                        toberemoved.push(fd);
                    }
                }

            }
            for t in toberemoved {
               self.fds.remove(&t).unwrap();
            }
            // demux
        }
        // loop
        drop(tx);
        let _ = executor_thread.join();
    }


}
