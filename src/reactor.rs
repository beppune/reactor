use std::collections::HashMap;
use std::vec;
use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};
use std::io::{self};

use nix::fcntl::{self, OFlag};
use nix::libc::O_RDONLY;
use nix::{poll::{PollFd, PollFlags, PollTimeout}, sys::stat::Mode};

enum Interest {
    Read,
}

pub enum Action {
    Stop, 
}

pub trait Handler {
    fn handle(&mut self, fd: OwnedFd) -> Action;
}

struct FileReadHandler {
    buffer: Vec<u8>,
}

impl Handler for FileReadHandler {
    fn handle(&mut self, fd: OwnedFd) -> Action {
        self.buffer.clear();
        let res = nix::unistd::read(fd, &mut self.buffer);
        let _ = dbg!(res);
        dbg!(&self.buffer);
        Action::Stop        
    }
}

pub struct Reactor {
    fds: HashMap<RawFd, (OwnedFd, Box<dyn Handler>, Interest)>,
}

impl Reactor {

    pub fn new() -> Self {
        Self {
            fds: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        // poll
        let mut topolls:Vec<RawFd> = vec![];
        {
            let mut p:Vec<PollFd> = self.fds.values().map(|(ofd, _, _)| PollFd::new( ofd.as_fd(), PollFlags::POLLIN) ).collect();
            let n = nix::poll::poll(&mut p, PollTimeout::NONE).unwrap();
            if n > 0 {
                topolls = p.iter().map(|k|k.as_fd().as_raw_fd()).collect();
            }
        }
        // demux
        for fd in topolls {
            let (ofd, mut hnd, _int) = self.fds.remove(&fd).unwrap();
            let action = hnd.handle(ofd);

            match action {
                Action::Stop => println!("Stop"),
            }
        }
    }

    pub(crate) fn read_file(&mut self, path:&str) -> io::Result<()> {
        let ofd = fcntl::open(path, OFlag::from_bits(O_RDONLY).unwrap(), Mode::empty())?;
        let fd = ofd.as_raw_fd();

        let h = Box::new(FileReadHandler {
            buffer: vec![]
        });

        self.fds.insert(fd, (ofd, h, Interest::Read) );

        Ok(())
    }

}
