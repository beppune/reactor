use std::cell::OnceCell;


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

struct Scope<'scoped> {
    handler: Vec<Box<dyn FnMut() + 'scoped>>
}

impl<'scoped> Scope<'scoped> {
    fn new() -> Scope<'scoped> {
        Scope { handler: vec![] }
    }

    fn add<F>(&mut self, f:F)
        where F:  FnMut() + 'scoped
    {
        self.handler.push( Box::new( f ) );
    }

    fn accept<F>(&mut self, address:&str, f:F)
        where F: FnMut(u32) + 'scoped
    {
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
