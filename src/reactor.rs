use std::cell::OnceCell;


struct Reactor {
}

impl Reactor {
    fn new() -> Self {
        Self {}
    }

    fn run<'scoped,F>(&mut self, mut f:F)
        where F: FnMut(&mut Scope) + 'scoped
    {
        let mut scope = Scope::new();
        f(&mut scope);

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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut reactor = Reactor::new();

        reactor.run(|scope|{
            scope.handler.push(Box::new(
                    ||{println!("Hello")}
            ));
        });

        assert!(true)
    }
}
