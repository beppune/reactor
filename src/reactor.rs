use std::{cell::OnceCell, collections::VecDeque, net::{TcpListener, TcpStream}, rc::Rc, sync::RwLock};
use std::io::Result;

use slab::Slab;


struct Reactor {
}

impl Reactor {
    fn new() -> Self {
        Self {}
    }

    fn run<'scoped,F>(&mut self, setup:F)
        where F: FnOnce(&mut Scope) + 'scoped
    {
        let mut scope = Scope::new();
        setup(&mut scope);

        for s in &mut scope.handler {
            s();
        }
    }
}

enum Resource {
    Listener(TcpListener),
}

enum Dispatch {
    Accept(usize)
}

struct Scope<'scoped> {
    handler: Vec<Box<dyn FnMut() + 'scoped>>,
    resources: Slab<Resource>,
    queue: Rc<Rwlock<VecDeque<Dispatch>>>,
}

impl<'scoped> Scope<'scoped> {
    fn new() -> Scope<'scoped> {
        Scope {
            handler: vec![],
            resources: Slab::new(),
            queue: Rc::new(RwLock::new(VecDeque::new())),
        }
    }

    fn add<F>(&mut self, f:F)
        where F:  FnMut() + 'scoped
    {
        self.handler.push( Box::new( f ) );
    }

    fn accept<F>(&mut self, address:&str, f:F) -> Result<usize>
        where F: FnMut(u32) + 'scoped
    {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        let res = self.resources.insert(Resource::Listener(listener));

        

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test() {
        let mut reactor = Reactor::new();
        let s = Rc::new(RefCell::new(String::from("Hello")));

        reactor.run(|scope|{

            let rs = s.clone();
            scope.add( move || {
                let mut ss = rs.borrow_mut();
                ss.push_str(" goodbye");
            } );

            scope.accept( "localhost:3113", |stream| {
            });
        });

        println!("{}", s.borrow());

        assert!(true)
    }
}
