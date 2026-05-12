
use std::os::fd::BorrowedFd;

use nix::sys::{signal::SigSet, signalfd::SignalFd};

use crate::{handler::{Action, Handler}, reactor::Reactor};

pub struct SignalHandler {
    sfd: SignalFd,
    callback: Option<Box<dyn FnOnce() + Send>>
}

impl Handler for SignalHandler {
    fn handle(&mut self, _fd: BorrowedFd) -> crate::handler::Action {

        let cb = std::mem::take(&mut self.callback).unwrap();

        (cb)();

        Action::Stop
    }
}


