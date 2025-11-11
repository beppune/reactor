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
    Accept(usize),
}

struct Scope<'scoped> {
    handler: Vec<Box<dyn FnMut() + 'scoped>>,
    resources: Rc<RwLock<Slab<Resource>>>,
    queue: Rc<RwLock<VecDeque<Dispatch>>>,
}

type Resources = Rc<RwLock<Slab<Resource>>>;
type Queue = Rc<RwLock<VecDeque<Dispatch>>>;

impl<'scoped> Scope<'scoped> {
    fn new() -> Scope<'scoped> {
        Scope {
            handler: vec![],
            resources: Rc::new(RwLock::new(Slab::new())),
            queue: Rc::new(RwLock::new(VecDeque::new())),
        }
    }

    fn add<F>(&mut self, f:F)
        where F:  FnMut() + 'scoped
    {
        self.handler.push( Box::new( f ) );
    }

    fn accept<F>(&mut self, address:&str, mut complete:F) -> Result<usize>
        where F: FnMut(Resources, usize) -> Option<Dispatch> + 'scoped
    {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        let res = self.resources.clone().write()
            .unwrap().insert(Resource::Listener(listener));

        let opt_dispatch = complete(self.resources.clone(), res);

        if let Some(dispatch) = opt_dispatch {
            self.queue.clone().write().unwrap().push_back( dispatch );
        }

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

            scope.accept( "localhost:3113", |tokens, token| {
                let res_listener = tokens.write().unwrap().remove(token);
                match res_listener {
                    Resource::Listener(listener) => {
                        match listener.accept() {
                            Ok((stream, addr)) => {
                                
                            },
                            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => todo!(),
                            Err(_) => todo!(),
                        }
                    }
                    _ => (),
                }

                None
            }).unwrap();
        });

        println!("{}", s.borrow());

        assert!(true)
    }
}
