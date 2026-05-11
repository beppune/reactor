use std::os::fd::BorrowedFd;

use nix::errno::Errno;

use crate::handler::*;

pub struct FileWriterHandler {
    pub buffer: Vec<u8>,
    pub complete: Option<Box<dyn FnOnce(Vec<u8>, usize)>>,
}

impl Handler for FileWriterHandler {
    fn handle(&mut self, fd: BorrowedFd) -> Action {
        let action:Action;

        match nix::unistd::write(fd, &mut self.buffer) {
            Ok(n) => {
                let data = std::mem::take(&mut self.buffer);
                let cb = std::mem::take(&mut self.complete).unwrap();

                (cb)(data, n);
                action = Action::Stop;
            },
            Err(e) if e == Errno::EAGAIN => action = Action::Continue,
            Err(_) => action = Action::Stop,
        }

        action
    }
}

pub struct FileReadHandler {
    pub buffer: Vec<u8>,
    pub complete: Option<Box<dyn FnOnce(Vec<u8>, usize)>>,
}

impl Handler for FileReadHandler {
    fn handle(&mut self, fd: BorrowedFd) -> Action {
        let action;
        match nix::unistd::read(fd, &mut self.buffer) {
            Ok(n) => {
                let data = std::mem::take(&mut self.buffer);
                let cb = std::mem::take(&mut self.complete).unwrap();

                (cb)(data, n);

                action = Action::Stop
            },
            Err(e) if e == Errno::EAGAIN => action = Action::Continue,
            Err(_) => action = Action::Stop,
        }

        action
    }
}
