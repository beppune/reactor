use std::{cell::RefCell, collections::VecDeque, io::Error, net::{TcpListener, TcpStream}, rc::Rc};

use slab::Slab;

use std::io::Result;

pub enum Nothing {}

pub enum Event {
    Accept(usize),
}

pub struct Token(usize);

enum Resource {
    Stream(TcpStream),
    Listener(TcpListener),
}

pub struct Scope<'scope>{
    handler: Vec<(Token,Box<dyn Fn(Token, Option<Error>) + 'scope>)>,
    resources: Slab<Resource>,
    queue: VecDeque<Event>,
}

impl<'scope> Scope<'scope> {

    fn new() -> Self {
        Self {
            handler: vec![],
            resources: Slab::new(),
            queue: VecDeque::new(),
        }
    }

    fn accept<Complete>(&mut self, addr:&str, handler:Complete) -> Result<()>
        where Complete: Fn(Token, Option<Error>) -> Option<Event> + 'scope
    {

        let listener = TcpListener::bind(addr)?;
        
        let index = self.resources.insert(Resource::Listener(listener));
        self.handler.push( (Token(index), Box::new(handler)) );

        Ok(())
    }
}

pub struct Reactor {}

impl Reactor {
    fn new() -> Self {
        Self {}
    }

    fn run_with<'scope, Setup>(&mut self, setup:Setup)
        where Setup: FnOnce(&mut Scope) -> Result<()> + 'scope
    {
        let mut scope = Scope::new();
        setup(&mut scope);
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

                Some(Event::Accept(index))

            })?;

            Ok(())
        });

        assert!(true);
    }
}


