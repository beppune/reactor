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
    }
}

struct Scope<'scoped> {
    handler: OnceCell<Box<dyn FnMut(&mut Scope) + 'scoped>>
}

impl<'scoped> Scope<'scoped> {
    fn new() -> Scope<'scoped> {
        Scope { handler: OnceCell::new() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut reactor = Reactor::new();

        reactor.run(|scope|{
            println!("Hello!");
        });

        assert!(true)
    }
}
