use std::sync::{Arc, Mutex};
use std::{os::fd::BorrowedFd};
use std::io;

use nix::{errno::Errno, fcntl::OFlag, libc::{O_CREAT, O_RDONLY, O_WRONLY}, sys::stat::Mode};
use nix::unistd::write;
use nix::unistd::read;

use nix::fcntl::open;

use crate::filectx::FileReadContext;
use crate::{handler::*, reactor::Reactor};

struct FileWriterHandler {
    pub buffer: Vec<u8>,
    pub complete: Arc<Mutex<dyn FnMut(Vec<u8>, usize) + Send>>,
    pub max: usize,
    pub offset: usize,
}

impl Handler for FileWriterHandler {
    fn handle(&mut self, fd: BorrowedFd) -> Action {
        let action:Action;

        let diff = self.buffer.len() - self.offset;
        let m = if diff <= self.max {
            self.offset + diff
        } else {
            self.offset + self.max
        };

        match write(fd, &self.buffer[self.offset..m]) {
            Ok(0) => action = Action::Stop,
            Ok(n) => {
                let m = self.offset + n;
                let chunk = self.buffer[self.offset..m].to_vec();
                let arc = self.complete.clone();

                let task = Box::new(move || {
                    let mut callback = arc.lock().unwrap();
                    (callback)(chunk, n);
                });

                action = Action::Task(task);
                self.offset = self.offset + n;
            },
            Err(e) if e == Errno::EAGAIN => action = Action::Continue,
            Err(_) => action = Action::Stop,
        }

        action
    }
}

struct FileReadHandler {
    pub buffer: Vec<u8>,
    pub ctx: FileReadContext,
}

impl Handler for FileReadHandler {
    fn handle(&mut self, fd: BorrowedFd) -> Action {
        let action;
        match read(fd, &mut self.buffer) {
            Ok(0) => {
                let eof = std::mem::take(&mut self.ctx.make_eof_task());
                if let Some(task) = eof {
                    return Action::TaskAndStop(task);
                }
                action = Action::Stop;
            },
            Ok(n) => {
                let chunk = self.buffer[..n].to_vec();
                let arc = self.ctx.make_chunk_task(chunk);

                let task = Box::new(move || {
                    let callback = arc.unwrap();
                    (callback)();
                });

                action = Action::Task(task);
            },
            Err(e) if e == Errno::EAGAIN => action = Action::Continue,
            Err(_) => action = Action::Stop,
        }

        action
    }
}

pub trait FileOperation {
    
    fn read_file(&mut self, path:&str, configure: impl FnOnce(&mut FileReadContext)) -> io::Result<()>;
    fn write_file(&mut self, path: &str, max:Option<usize>, buffer:Vec<u8>, cb: impl FnMut(Vec<u8>, usize) + Send + 'static ) -> io::Result<()>;
}

impl FileOperation for Reactor {
    
    fn read_file(&mut self, path:&str, configure: impl FnOnce(&mut FileReadContext)) -> io::Result<()> {
        let ofd = open(path, OFlag::from_bits(O_RDONLY).unwrap(), Mode::empty())?;

        let mut ctx = FileReadContext::new(512);

        configure(&mut ctx);

        let h = Box::new(FileReadHandler {
            buffer: vec![0; ctx.chunk_size()],
            ctx,
        });

        self.register(ofd, h, Interest::Read);

        Ok(())
    }

    fn write_file(&mut self, path: &str, max:Option<usize>, buffer:Vec<u8>, cb: impl FnMut(Vec<u8>, usize) + Send + 'static ) -> io::Result<()> {

        let ofd = open(path, OFlag::from_bits(O_WRONLY|O_CREAT).unwrap(), Mode::empty())?;

        let m = max.unwrap_or(512);

        let h = Box::new(FileWriterHandler {
            buffer,
            offset: 0,
            max: m,
            complete: Arc::new(Mutex::new(cb)),
        });

        self.register(ofd, h, Interest::Write);

        Ok(())
    }
}
