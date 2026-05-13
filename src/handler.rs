use std::os::fd::BorrowedFd;

pub enum Interest {
    Read,
    Write,
}

pub enum Action {
    Stop,
    Continue, 
    Task(Box<dyn FnOnce() + Send + 'static>),
    TaskAndStop(Box<dyn FnOnce() + Send + 'static>),
}

pub trait Handler {
    fn handle(&mut self, fd: BorrowedFd) -> Action;
}
