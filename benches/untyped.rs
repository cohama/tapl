
#![feature(test)]

extern crate tapl;
extern crate test;

use tapl::untyped::*;
use tapl::untyped::Term::*;

use std::rc::Rc;


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    fn abs(name: &str, t: Term) -> Term {
        Abs(name.to_string(), Rc::new(t))
    }

    fn app(t1: Term, t2: Term) -> Term {
        App(Rc::new(t1), Rc::new(t2))
    }

    #[bench]
    fn term_eval(b: &mut Bencher) {
        // (λf. λx. f x) (λz. λw. z) (λu. u) => λw. λu. u
        let t = app(app(abs("f", abs("x", app(Var(1, 2), Var(0, 2)))), abs("z", abs("w", Var(1, 2)))), abs("u", Var(0, 1))).eval();
        b.iter(|| t.eval())
    }
}
