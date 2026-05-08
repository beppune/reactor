use std::vec;
use std::os::fd::{AsFd, OwnedFd};
use std::io;
use std::fs::File;

use nix::poll::{self, PollFd};

enum Interest {
    Read,
}

enum Action {
    Stop, 
}

pub trait Handler {
    fn handle(&mut self) -> Action;
}

struct FileReadHandler {
    fd: OwnedFd,
    content: String,
    complete: Box<dyn Fn(&String)->Action>,
}

impl Handler for FileReadHandler {
    fn handle(&mut self) -> Action {
        
        (self.complete)(&self.content)
    }
}

pub struct Reactor {
    pollable: Vec<(Box<dyn Handler>, Interest)>
}

impl Reactor {

    pub fn new() -> Self {
        Self {
            pollable: vec![],
        }
    }

    pub fn run(&mut self) {
        // poll
        let mut polls: Vec<PollFd> = vec![];
        for i in self.pollable {
            polls.push(i.0.fd);
        }

        // demux

        // update
    }

    pub(crate) fn read_file(&mut self, path:&str, complete:impl Fn(&String)->Action + 'static) -> io::Result<()> {
        let fd = File::open(path)?
            .as_fd().try_clone_to_owned()?;

        let h = Box::new(FileReadHandler {
            fd,
            content: String::new(),
            complete: Box::new(complete),
        });

        self.pollable.push((h, Interest::Read));

        Ok(())
    }

}
