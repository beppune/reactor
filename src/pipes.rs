use std::{io, os::fd::BorrowedFd};

use nix::{fcntl::OFlag, libc::{O_NONBLOCK, O_RDONLY, read}, sys::stat::Mode};

use crate::{framer::Framer, handler::{Handler, Interest}, reactor::Reactor};


struct PipeReadHandler {
}

impl Handler for PipeReadHandler {
    fn handle(&mut self, fd: BorrowedFd) -> crate::handler::Action {
        todo!();
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

        Ok(())
    }
}
