
struct Reactor {
}

impl Reactor {
    fn new() -> Self {
        Self {}
    }

    fn run<'scoped,F>(&mut self, mut f:F)
        where F: FnMut() + 'scoped
    {
        f();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut reactor = Reactor::new();

        reactor.run(||{
            println!("Hello!");
        });

        assert!(true)
    }
}
