use std::{collections::VecDeque, io, os::fd::BorrowedFd, sync::{Arc, Mutex}, vec};

use nix::{fcntl::OFlag, libc::{O_NONBLOCK, O_RDONLY, read}, sys::stat::Mode, unistd::read};

use crate::{framer::Framer, handler::{Handler, Interest}, reactor::Reactor};


struct PipeReadHandler {
    pub buffer: VecDeque<u8>,
    pub complete: Arc<Mutex<dyn FnMut(Vec<u8>) + Send>>,
}

impl Handler for PipeReadHandler {
    fn handle(&mut self, fd: BorrowedFd) -> crate::handler::Action {
        match read(fd, self.buffer.get_mut) {
            
        }
    }
}

trait PipeOperations {
    fn read_named_pipe(&mut self, path: &str, cb: impl FnMut(Vec<u8>) + Send + 'static) -> io::Result<()>;
}

impl PipeOperations for Reactor {
    fn read_named_pipe(&mut self, path: &str, cb: impl FnMut(Vec<u8>) + Send + 'static) -> io::Result<()> {
        let oflags = OFlag::from_bits(O_NONBLOCK| O_RDONLY).unwrap();
        let mode = Mode::empty();
        let ofd = nix::fcntl::open(path, oflags, mode)?;

        let h = Box::new(PipeReadHandler {
            buffer: VecDeque::with_capacity(512),
            complete: Arc::new(Mutex::new(cb)),
        });

        self.register(ofd, h, Interest::Read);

        Ok(())
    }
}
