use std::{cell::RefCell, collections::VecDeque, io::{Error, ErrorKind}, net::{TcpListener, TcpStream}, rc::Rc};

use slab::Slab;

use std::io::Result;

pub enum Nothing {}

pub enum Event {
    Accept(usize),
    // Write(usize),
    // Read(usize),
    Stop,
}

pub struct Token(usize);

enum Resource {
    Stream(TcpStream),
    Listener(TcpListener),
}

pub struct Scope<'scope>{
    handler: Vec<(Token,Box<dyn Fn(Token, Option<Error>) -> Option<Event> + 'scope>)>,
    resources: Rc<RefCell<Slab<Resource>>>,
    queue: VecDeque<Event>,
}

impl<'scope> Scope<'scope> {

    fn new() -> Self {
        Self {
            handler: vec![],
            resources: Rc::new(RefCell::new(Slab::new())),
            queue: VecDeque::new(),
        }
    }

    fn accept<Complete>(&mut self, addr:&str, handler:Complete) -> Result<()>
        where Complete: Fn(Token, Option<Error>) -> Option<Event> + 'scope
    {

        let listener = TcpListener::bind(addr)?;

        listener.set_nonblocking(true)?;
        
        let index = self.resources.borrow_mut().insert(Resource::Listener(listener));

        self.queue.push_back( Event::Accept(index) );
        self.handler.push( (Token(index), Box::new(handler)) );

        Ok(())
    }
}

pub struct Reactor {}

impl Reactor {
    fn new() -> Self {
        Self {}
    }

    fn poll(&mut self, scope: &mut Scope) -> bool {
        for res in scope.resources.borrow_mut().iter_mut() {
            match res {
                (_, Resource::Listener(listener) ) => {
                    match listener.accept() {
                        Ok( (stream, _endpoint) ) => {
                            let index = scope.resources.borrow_mut().insert( Resource::Stream(stream) );
                            scope.queue.push_back( Event::Accept(index) );
                        },
                        Err(err) if err.kind() == ErrorKind::WouldBlock => { /*do nothing*/ },
                        Err(err) => println!("Accept: {err}"),
                    }
                    return false;
                },
                _ => {},
            }
        }
        return true;
    }

    fn demux(&mut self, scope:&mut Scope) -> bool {
        if let Some(event) = scope.queue.pop_front() {
            match event {
                Event::Accept(index) => {
                    if let Some(_stream) = scope.resources.borrow_mut().get_mut(index) {
                        let found = scope.handler.iter().find( |t| matches!(t.0, Token(i) if i == index) );
                        if let Some((_, handler)) = found {
                            let optev = handler(Token(index), None);

                            if let Some(ev) = optev {
                                scope.queue.push_back( ev );
                            }
                        }
                    }

                },
                Event::Stop => {
                    return true;
                }
            }
        }
        return false;
    }

    fn run_with<'scope, Setup>(&mut self, setup:Setup)
        where Setup: FnOnce(&mut Scope) -> Result<()> + 'scope
    {
        let mut scope = Scope::new();
        if let Err(error) = setup(&mut scope) {
            println!("Setup: {error}");
            return;
        }

        loop{
            scope.resources.borrow_mut().shrink_to_fit();
            while self.poll(&mut scope) {}
            if self.demux(&mut scope) {
                break;
            }
        }

    }
}

#[cfg(test)]
mod test {
    use std::{io::Write, net::Shutdown};

    use super::*;
    #[test]
    fn test() {
        let mut reactor = Reactor::new();

        reactor.run_with(|scope|{
            scope.accept("localhost:31313", |token, opterr|{
                if let Some(err) = opterr {
                    println!("{err}");
                    return None;
                }

                let Token(index) = token;
                println!("Do somethingwith {index}");
                // Some(Event::Stop)
                None
            })?;

            Ok(())
        });

        assert!(true);
    }
}


