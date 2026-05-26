use std::{collections::VecDeque, io, os::fd::BorrowedFd, sync::{Arc, Mutex}, vec};

use nix::{errno::Errno, fcntl::OFlag, libc::{O_NONBLOCK, O_RDONLY, O_WRONLY}, sys::stat::Mode };

use crate::{framer::{Buffer, Framer}, handler::{Action, Handler, Interest}, reactor::{self, Reactor, ReactorHandle}};

#[derive(Clone)]
pub struct PipeContext {
    pub buffer: Arc<Mutex<VecDeque<u8>>>,
    pub on_chunk: Arc<Mutex<Option<Box<dyn FnMut(Vec<u8>, &PipeContext) + Send>>>>,
    pub on_close: Arc<Mutex<Option<Box<dyn FnOnce(&PipeContext) + Send>>>>,
    pub reactor: ReactorHandle,
}

impl PipeContext {
    fn new(buf_size: usize, reactor: ReactorHandle) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(buf_size))),
            on_chunk: Arc::new(Mutex::new(None)),
            on_close: Arc::new(Mutex::new(None)),
            reactor,
        }
    }

    pub fn make_chunk_task(&self, chunk: Vec<u8>) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        let ctx = self.clone();

        Some(Box::new(move || {
            let mut slot = ctx.on_chunk.lock().unwrap();

            if let Some(cb) = slot.as_mut() {
                (cb)(chunk, &ctx);
            } 
        }))
    }

    pub fn on_chunk(&mut self, cb: impl FnMut(Vec<u8>, &PipeContext) + Send + 'static) {

        *self.on_chunk.lock().unwrap() = Some(Box::new(cb));

    }

    pub fn make_close_task(&self) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        let ctx = self.clone();

        Some(Box::new( move || {
            let mut slot = ctx.on_close.lock().unwrap();

            if let Some(cb) = slot.take() {
                (cb)(&ctx);
            }
        }))
    }

    pub fn on_close(&mut self, cb: impl FnOnce(&PipeContext) + Send + 'static) {
        *self.on_close.lock().unwrap() = Some(Box::new(cb));
    }


}

struct PipeReadHandler {
    pub buffer: Buffer,
    pub temp: Vec<u8>,
    pub ctx: PipeContext,
}

impl Handler for PipeReadHandler {
    fn handle(&mut self, fd: BorrowedFd) -> crate::handler::Action {
        let action: Action;
        match nix::unistd::read(fd, &mut self.temp) {
            Ok(0) => {

                while self.buffer.next_frame().is_some() {
                    //do nothing
                }

                let arc = self.ctx.make_close_task();

                if let Some(callback) = arc {
                    let task = Box::new(move || {
                        (callback)();
                    });

                    action = Action::TaskAndStop(task);
                } else {
                    action = Action::Stop;
                }

            },
            Ok(n) => {
                if let Err(_e) = self.buffer.push(&self.temp[..n]) {
                    action = Action::Stop;
                    return action;
                }
                let frame = self.buffer.next_frame();
                if frame.is_none() {
                    action = Action::Continue;
                    return action;
                }
                let chunk = frame.unwrap();
                let arc = self.ctx.make_chunk_task(chunk);

                if let Some(callback) = arc {

                    let task = Box::new(move || {
                        (callback)();
                    });

                    action = Action::Task(task)
                } else {
                    action = Action::Continue;
                }

                let _ = self.ctx.reactor.sender.send(Box::new(|_r: &mut Reactor|{
                    println!("I'm the reactor");
                }));

            },
            Err(Errno::EAGAIN) => action = Action::Continue,
            Err(e) => {
                println!("pipe read: {}", e);
                while self.buffer.next_frame().is_some() {
                    // do nothing
                }
                action = Action::Stop;
            }
        }

        action
    }
}

pub struct PipeWriteHadler {
    pub temp: Vec<u8>,
    pub ctx: PipeContext,
}

impl Handler for PipeWriteHadler {
    fn handle(&mut self, fd: BorrowedFd) -> crate::handler::Action {
        let action: Action;
        match nix::unistd::write(fd, &self.temp[..self.temp.len()]) {
            Ok(0) => {

                let arc = self.ctx.make_close_task();
                if let Some(callback) = arc { let task = Box::new(move || {
                    (callback)();
                });

                    action = Action::TaskAndStop(task);
                } else {
                    action = Action::Stop;
                }

            },
            Ok(_n) => {
                let chunk = self.temp.clone();
                let arc = self.ctx.make_chunk_task(chunk);

                if let Some(callback) = arc {

                    let task = Box::new(move || {
                        (callback)();
                    });

                    action = Action::Task(task)
                } else {
                    action = Action::Continue;
                }

            },
            Err(Errno::EAGAIN) => action = Action::Continue,
            Err(e) => {
                println!("pipe read: {}", e);
                action = Action::Stop;
            }
        }

        action
    }
}

pub trait PipeOperations {
    fn read_named_pipe(&mut self, path: &str, cb: impl FnOnce(&mut PipeContext)) -> io::Result<()>;
    fn write_named_pipe(&mut self, buffer:Vec<u8>, path: &str, cb: impl FnOnce(&mut PipeContext)) -> io::Result<()>;
}

impl PipeOperations for Reactor {
    fn read_named_pipe(&mut self, path: &str, config: impl FnOnce(&mut PipeContext)) -> io::Result<()> {
        let oflags = OFlag::from_bits(O_NONBLOCK| O_RDONLY).unwrap();
        let mode = Mode::empty();
        let ofd = nix::fcntl::open(path, oflags, mode)?;

        let reactor = ReactorHandle {
            sender: self.command_sender.clone(),
        };

        let mut ctx = PipeContext::new(512, reactor);

        (config)(&mut ctx);

        let max = ctx.buffer.lock().unwrap().capacity();

        let temp:Vec<u8> = vec![0; max];

        let h = Box::new(PipeReadHandler {
            buffer: Buffer::new(512),
            temp,
            ctx,
        });

        self.register(ofd, h, Interest::Read);

        Ok(())
    }

    fn write_named_pipe(&mut self, buffer:Vec<u8>, path: &str, config: impl FnOnce(&mut PipeContext)) -> io::Result<()> {
        let oflags = OFlag::from_bits(O_NONBLOCK| O_WRONLY).unwrap();
        let mode = Mode::empty();
        let ofd = nix::fcntl::open(path, oflags, mode)?;

        let reactor = ReactorHandle {
            sender: self.command_sender.clone(),
        };

        let mut ctx = PipeContext::new(512, reactor);

        (config)(&mut ctx);

        let h = Box::new(PipeWriteHadler{
            temp: buffer,
            ctx,
        });

        self.register(ofd, h, Interest::Write);

        Ok(())
    }


}
