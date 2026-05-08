use std::collections::HashMap;
use std::vec;
use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};
use std::io::{self, ErrorKind};
use std::fs::File;

use nix::fcntl::{self, OFlag};
use nix::libc::O_RDONLY;
use nix::poll::{self, PollFd, PollFlags};
use nix::sys::stat::Mode;

enum Interest {
    Read,
}

enum Action {
    Stop, 
}

pub trait Handler {
    fn handle(&mut self, fd: RawFd) -> Action;
}

struct FileReadHandler {
    buffer: Vec<u8>,
    ofd: OwnedFd,
}

impl Handler for FileReadHandler {
    fn handle(&mut self, fd: RawFd) -> Action {

        Action::Stop        
    }
}

pub struct Reactor {
    fds: HashMap<RawFd, Box<dyn Handler>>,
}

impl Reactor {

    pub fn new() -> Self {
        Self {
            fds: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        // poll
        self.fds.keys().map(|f| {
            
        });

        // demux

        // update
    }

    pub(crate) fn read_file(&mut self, path:&str) -> io::Result<()> {
        let ofd = fcntl::open(path, OFlag::from_bits(O_RDONLY).unwrap(), Mode::empty())?;
        let fd = ofd.as_raw_fd();

        let h = Box::new(FileReadHandler {
            ofd,
            buffer: vec![]
        });

        self.fds.insert(fd, h);

        Ok(())
    }

}
