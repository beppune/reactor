use std::os::fd::BorrowedFd;

pub enum Interest {
    Read,
    Write,
}

pub enum Action {
    Stop,
    Continue, 
}

pub trait Handler {
    fn handle(&mut self, fd: BorrowedFd) -> Action;
}
