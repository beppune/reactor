use std::{io, os::fd::{BorrowedFd, OwnedFd}, time::Duration};

use nix::{libc::TFD_NONBLOCK, sys::{time::TimeSpec, timerfd::{Expiration, TimerFd, TimerFlags, TimerSetTimeFlags}}};
use nix::sys::timerfd::ClockId;

use crate::{handler::{Action, Handler, Interest}, reactor::Reactor};


pub struct TimerHander {
    callback: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl Handler for TimerHander {
    fn handle(&mut self, _fd: BorrowedFd) -> crate::handler::Action {
        let cb = std::mem::take(&mut self.callback).unwrap();

        (cb)();

        Action::Stop
    }
}

pub trait TimerOperation {
    fn start_timer(&mut self, d:Duration, cb: impl FnOnce() + Send + 'static ) -> io::Result<()>;
}

impl TimerOperation for Reactor {
    fn start_timer(&mut self, d:Duration, cb: impl FnOnce() + Send + 'static ) -> io::Result<()> {

        let tfd = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::from_bits(TFD_NONBLOCK).unwrap() )?;
        tfd.set(Expiration::OneShot(TimeSpec::from_duration(d)), TimerSetTimeFlags::empty())?;

        let h = Box::new(TimerHander {
            callback: Some(Box::new(cb)),
        });

        self.register(OwnedFd::from(tfd), h, Interest::Read);

        Ok(())
    }
}
