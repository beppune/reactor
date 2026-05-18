use std::{collections::VecDeque, io, os::fd::BorrowedFd, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, vec};

use nix::{errno::Errno, fcntl::OFlag, libc::{CBAUD, O_NONBLOCK, O_RDONLY}, sys::stat::Mode };

use crate::{framer::Framer, handler::{Action, Handler, Interest}, reactor::Reactor};

#[derive(Clone)]
pub struct PipeContext {
    pub buffer: Arc<Mutex<VecDeque<u8>>>,
    pub on_chunk: Arc<Mutex<Option<Box<dyn FnMut(Vec<u8>, &PipeContext) + Send>>>>,
    pub on_close: Arc<Mutex<Option<Box<dyn FnOnce(&PipeContext) + Send>>>>,
}

impl PipeContext {
    fn new(buf_size: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(buf_size))),
            on_chunk: Arc::new(Mutex::new(None)),
            on_close: Arc::new(Mutex::new(None)),
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
    pub temp: Vec<u8>,
    pub ctx: PipeContext,
}

impl Handler for PipeReadHandler {
    fn handle(&mut self, fd: BorrowedFd) -> crate::handler::Action {
        let action: Action;
        match nix::unistd::read(fd, &mut self.temp) {
            Ok(0) => {
                let arc = self.ctx.make_close_task();

                let task = Box::new(move || {
                    let callback = arc.unwrap();
                    (callback)();
                });

                action = Action::TaskAndStop(task);

            },
            Ok(n) => {
                let chunk = self.temp[..n].to_vec();
                let arc = self.ctx.make_chunk_task(chunk);

                let task = Box::new(move || {
                    let callback = arc.unwrap();
                    (callback)();
                });

                action = Action::Task(task);
            },
            Err(Errno::EAGAIN) => action = Action::Continue,
            Err(_) => action = Action::Stop,
        }

        action
    }
}

pub trait PipeOperations {
    fn read_named_pipe(&mut self, path: &str, cb: impl FnOnce(&mut PipeContext)) -> io::Result<()>;
}

impl PipeOperations for Reactor {
    fn read_named_pipe(&mut self, path: &str, config: impl FnOnce(&mut PipeContext)) -> io::Result<()> {
        let oflags = OFlag::from_bits(O_NONBLOCK| O_RDONLY).unwrap();
        let mode = Mode::empty();
        let ofd = nix::fcntl::open(path, oflags, mode)?;

        let mut ctx = PipeContext::new(512);

        (config)(&mut ctx);

        let max = ctx.buffer.lock().unwrap().capacity();

        let temp:Vec<u8> = vec![0; max];

        let h = Box::new(PipeReadHandler {
            temp,
            ctx,
        });

        self.register(ofd, h, Interest::Read);

        Ok(())
    }
}
