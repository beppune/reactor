use std::collections::HashMap;

use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};
use std::io::{self};

use nix::fcntl::{self, OFlag};
use nix::libc::{O_CREAT, O_RDONLY, O_WRONLY, POLLIN, POLLOUT};
use nix::{poll::{PollFd, PollFlags, PollTimeout}, sys::stat::Mode};

use crate::files::{FileReadHandler, FileWriterHandler};
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

    pub fn run(&mut self) {
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
                }
            }
            // demux
        }
        // loop
    }

    pub fn read_file(&mut self, path:&str, cb:impl FnOnce(Vec<u8>, usize) + 'static ) -> io::Result<()> {
        let ofd = fcntl::open(path, OFlag::from_bits(O_RDONLY).unwrap(), Mode::empty())?;
        let fd = ofd.as_raw_fd();

        let h = Box::new(FileReadHandler {
            buffer: vec![0; 512],
            complete: Some(Box::new(cb)),
        });

        self.fds.insert(fd, (ofd, h, Interest::Read) );

        Ok(())
    }

    pub fn write_file(&mut self, path: &str, buffer:Vec<u8>, cb: impl FnOnce(Vec<u8>, usize) + 'static ) -> io::Result<()> {

        let ofd = fcntl::open(path, OFlag::from_bits(O_WRONLY|O_CREAT).unwrap(), Mode::empty())?;

        let fd = ofd.as_raw_fd();

        let h = Box::new(FileWriterHandler {
            buffer,
            complete: Some(Box::new(cb)),
        });

        self.fds.insert(fd, (ofd, h, Interest::Write));

        Ok(())
    }

}
