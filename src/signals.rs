use std::io;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use nix::errno::Errno;
use nix::sys::signal::{self, SigSet, Signal};
use nix::sys::signalfd::{SignalFd, SfdFlags};

use crate::handler::{Handler, Action, Interest};
use crate::reactor::Reactor;

/// Handler for Linux signalfd (one-shot semantics)
pub struct SignalHandler {
    fd: SignalFd,
    callback: Option<Box<dyn FnOnce(Signal) + Send>>,
}

impl Handler for SignalHandler {
    fn handle(&mut self, _fd: BorrowedFd) -> Action {
        match self.fd.read_signal() {
            Ok(Some(info)) => {
                // One signal consumed
                let signo = Signal::try_from(info.ssi_signo as i32)
                    .unwrap_or(Signal::SIGTERM);

                if let Some(cb) = self.callback.take() {
                    let task = Box::new(move || {
                        cb(signo);
                    });
                    return Action::Task(task);
                }

                Action::Stop
            },

            Ok(None) => {
                // Non-blocking fd, no signal available
                Action::Continue
            },

            Err(e) if e == Errno::EAGAIN => Action::Continue,

            Err(_) => Action::Stop,
        }
    }
}

/// Extension trait for Reactor
pub trait SignalOperations {
    fn on_signal(
        &mut self,
        signals: &[Signal],
        callback: impl FnOnce(Signal) + Send + 'static,
    ) -> io::Result<()>;
}

impl SignalOperations for Reactor {
    fn on_signal(
        &mut self,
        signals: &[Signal],
        callback: impl FnOnce(Signal) + Send + 'static,
    ) -> io::Result<()> {
        // Build signal set
        let mut mask = SigSet::empty();
        for s in signals {
            mask.add(*s);
        }

        // Block signals so they are delivered via signalfd
        signal::sigprocmask(signal::SigmaskHow::SIG_BLOCK, Some(&mask), None)?;

        // Create signalfd (non-blocking)
        let sfd = SignalFd::with_flags(&mask, SfdFlags::SFD_NONBLOCK)?;

        let owned: OwnedFd = sfd.as_fd().try_clone_to_owned()?;

        let handler = Box::new(SignalHandler {
            fd: sfd,
            callback: Some(Box::new(callback)),
        });

        self.register(owned, handler, Interest::Read);
        Ok(())
    }
}
