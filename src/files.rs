use std::{os::fd::BorrowedFd};
use std::io;

use nix::{errno::Errno, fcntl::OFlag, libc::{O_CREAT, O_RDONLY, O_WRONLY}, sys::stat::Mode};
use nix::unistd::write;
use nix::unistd::read;

use std::mem::take;
use nix::fcntl::open;

use crate::{handler::*, reactor::Reactor};

struct FileWriterHandler {
    pub buffer: Vec<u8>,
    pub complete: Option<Box<dyn FnOnce(Vec<u8>, usize)>>,
}

impl Handler for FileWriterHandler {
    fn handle(&mut self, fd: BorrowedFd) -> Action {
        let action:Action;

        match write(fd, &mut self.buffer) {
            Ok(n) => {
                let data = take(&mut self.buffer);
                let cb = take(&mut self.complete).unwrap();

                (cb)(data, n);
                action = Action::Stop;
            },
            Err(e) if e == Errno::EAGAIN => action = Action::Continue,
            Err(_) => action = Action::Stop,
        }

        action
    }
}

struct FileReadHandler {
    pub buffer: Vec<u8>,
    pub complete: Option<Box<dyn FnOnce(Vec<u8>, usize)>>,
}

impl Handler for FileReadHandler {
    fn handle(&mut self, fd: BorrowedFd) -> Action {
        let action;
        match read(fd, &mut self.buffer) {
            Ok(n) => {
                let data = take(&mut self.buffer);
                let cb = take(&mut self.complete).unwrap();

                (cb)(data, n);

                action = Action::Stop
            },
            Err(e) if e == Errno::EAGAIN => action = Action::Continue,
            Err(_) => action = Action::Stop,
        }

        action
    }
}

pub trait FileOperation {
    
    fn read_file(&mut self, path:&str, cb:impl FnOnce(Vec<u8>, usize) + 'static ) -> io::Result<()>;
    fn write_file(&mut self, path: &str, buffer:Vec<u8>, cb: impl FnOnce(Vec<u8>, usize) + 'static ) -> io::Result<()>;
}

impl FileOperation for Reactor {
    
    fn read_file(&mut self, path:&str, cb:impl FnOnce(Vec<u8>, usize) + 'static ) -> io::Result<()> {
        let ofd = open(path, OFlag::from_bits(O_RDONLY).unwrap(), Mode::empty())?;

        let h = Box::new(FileReadHandler {
            buffer: vec![0; 512],
            complete: Some(Box::new(cb)),
        });

        self.register(ofd, h, Interest::Read);

        Ok(())
    }

    fn write_file(&mut self, path: &str, buffer:Vec<u8>, cb: impl FnOnce(Vec<u8>, usize) + 'static ) -> io::Result<()> {

        let ofd = open(path, OFlag::from_bits(O_WRONLY|O_CREAT).unwrap(), Mode::empty())?;

        let h = Box::new(FileWriterHandler {
            buffer,
            complete: Some(Box::new(cb)),
        });

        self.register(ofd, h, Interest::Write);

        Ok(())
    }
}
