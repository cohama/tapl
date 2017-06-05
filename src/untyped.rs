
use std::rc::Rc;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Term {
    Var(usize, usize),
    Abs(String, Rc<Term>),
    App(Rc<Term>, Rc<Term>)
}

#[derive(Debug, Clone)]
struct NameBind;

type Binding = NameBind; // naming context (名前付け文脈) p58 6.1

#[derive(Debug, Clone)]
pub struct Context(Vec<(String, Binding)>);

impl Context {
    pub fn new<T: AsRef<str>>(names: &[T]) -> Context {
        Context(names.iter().map(|name| (name.as_ref().to_string(), NameBind)).collect())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
    fn index_to_name(&self, n: usize) -> &String {
        // de Bruijn インデックスはラムダ抽象の内側から外側へ番号を振る。
        // しかし、pick_fresh_name により文脈が増加するときには内部の Vec
        // は後方に新しい変数を配置する。
        // つまり、最も新しい変数は Vec の最後方に位置することになる。
        // インデックスのアクセスを逆順にすることにより、小さい添字が
        // 新しい変数を表すようにする。
        &self.0[self.len() - 1 - n].0
    }
    fn pick_fresh_name(&self, name: &str) -> (Context, String) {
        let has = self.0.iter().any(|x| name == x.0);
        if has {
            self.pick_fresh_name(&format!("{}'", name))
        } else {
            let name = name.to_string();
            let mut newone = self.clone();
            newone.0.push((name.clone(), NameBind));
            (newone, name)
        }
    }
}

impl Term {
    pub fn show(&self, ctx: &Context) -> String {
        use self::Term::*;
        match *self {
            Var(x, n) => {
                if ctx.len() == n {
                    ctx.index_to_name(x).clone()
                } else {
                    panic!("bad index. Context len is {} but var has {}.", ctx.len(), n)
                }
            }
            Abs(ref name, ref t) => {
                let (ctx, name) = ctx.pick_fresh_name(name);
                format!("(λ{}. {})", name, t.show(&ctx))
            }
            App(ref t1, ref t2) => format!("({} {})", t1.show(ctx), t2.show(ctx))
        }
    }

    fn shift(&self, d: isize) -> Term {
        fn walk(t: &Term, d: isize, c: usize) -> Term {
            use self::Term::*;
            match *t {
                Var(x, n) => Var(if x >= c {(x as isize + d) as usize} else {x}, (n as isize + d) as usize),
                Abs(ref name, ref t) => Abs(name.clone(), Rc::new(walk(t, d, c+1))),
                App(ref t1, ref t2) => App(Rc::new(walk(t1, d, c)), Rc::new(walk(t2, d, c)))
            }
        }
        walk(self, d, 0)
    }

    fn subst(&self, j: usize, ts: &Term) -> Term {
        fn walk(t: &Term, j: usize, c: usize, ts: &Term) -> Term {
            use self::Term::*;
            match *t {
                Var(x, _) => if x == j+c {ts.shift(c as isize)} else {t.clone()},
                Abs(ref name, ref t) => Abs(name.clone(), Rc::new(walk(t, j, c+1, ts))),
                App(ref t1, ref t2) => App(Rc::new(walk(t1, j, c, ts)), Rc::new(walk(t2, j, c, ts)))
            }
        }
        walk(self, j, 0, ts)
    }

    fn subst_top(&self, s: &Term) -> Term {
        self.subst(0, &s.shift(1)).shift(-1)
    }

    fn is_val(&self) -> bool {
        match *self {
            Term::Abs(_, _) => true,
            _ => false
        }
    }

    fn eval1(&self) -> Result<Term, String> {
        use self::Term::*;
        if let App(ref t1, ref t2) = *self {
            match t1.as_ref() {
                &Abs(_, ref t12) if t2.is_val() => Ok(t12.subst_top(&t2)),
                v if v.is_val() => {
                    let t2 = t2.eval1()?;
                    Ok(App(Rc::new(v.clone()), Rc::new(t2)))
                }
                _ => {
                    let t1 = t1.eval1()?;
                    Ok(App(Rc::new(t1), t2.clone()))
                }
            }
        } else {
            Err(format!("No rule applies: {:?}", self))
        }
    }

    pub fn eval(&self) -> Term {
        match self.eval1() {
            Ok(t) => t.eval(),
            Err(_) => self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Term::*;

    fn abs(name: &str, t: Term) -> Term {
        Abs(name.to_string(), Rc::new(t))
    }

    fn app(t1: Term, t2: Term) -> Term {
        App(Rc::new(t1), Rc::new(t2))
    }

    #[test]
    fn term_show() {
        let context = Context::new(&["x"]);
        assert_eq!(Var(0, 1).show(&context), "x");
        assert_eq!(abs("y", Var(1, 2)).show(&context), "(λy. x)");
        assert_eq!(abs("x", Var(0, 2)).show(&context), "(λx'. x')");
        assert_eq!(abs("y", Var(0, 2)).show(&context), "(λy. y)");
        assert_eq!(abs("y", Var(1, 2)).show(&context), "(λy. x)");
        assert_eq! {
            app(
                abs("y", Var(1, 2)),
                abs("z", Var(0, 2)),
            ).show(&context),
            "((λy. x) (λz. z))"
        }
    }

    #[test]
    fn term_shift() {
        assert_eq! {
            // ↑2 (λ.λ. 1 (0 2)) => λ.λ. 1 (0 4)
            abs("a", abs("b", app(Var(1, 100), app(Var(0, 100), Var(2, 100))))).shift(2),
            abs("a", abs("b", app(Var(1, 102), app(Var(0, 102), Var(4, 102)))))
        }
        assert_eq! {
            // ↑2 (λ.0 1 (λ. 0 1 2)) => λ.0 3 (λ. 0 1 4)
            abs("a", app(app(Var(0, 100), Var(1, 100)), abs("b", app(app(Var(0, 100), Var(1, 100)), Var(2, 100))))).shift(2),
            abs("a", app(app(Var(0, 102), Var(3, 102)), abs("b", app(app(Var(0, 102), Var(1, 102)), Var(4, 102)))))
        }
    }

    #[test]
    fn term_subst() {
        assert_eq! {
            // [0 -> 1](0 λ.λ. 2) => 1 (λ.λ. 3)
            app(Var(0, 100), abs("x", abs("y", Var(2, 100)))).subst(0, &Var(1, 100)),
            app(Var(1, 100), abs("x", abs("y", Var(3, 102))))
        }
        assert_eq! {
            // [0 -> 1 λ. 2](0 λ. 1) => (1 λ.2) (λ. (2 λ. 3))
            app(Var(0, 100), abs("x", Var(1, 100))).subst(0, &app(Var(1, 100), abs("z", Var(2, 100)))),
            app(app(Var(1, 100), abs("z", Var(2, 100))), abs("x", app(Var(2, 101), abs("z", Var(3, 101)))))
        }
    }

    #[test]
    fn term_eval() {
        assert_eq! {
            // (λ. 0) (λ. 0) => λ. 0
            app(abs("x", Var(0, 2)), abs("y", Var(0, 1))).eval(),
            abs("y", Var(0, 1))
        }
        assert_eq! {
            // (λf. λx. f x) (λz. λw. z) (λu. u) => λw. λu. u
            app(app(abs("f", abs("x", app(Var(1, 2), Var(0, 2)))), abs("z", abs("w", Var(1, 2)))), abs("u", Var(0, 1))).eval(),
            abs("w", abs("u", Var(0, 2)))
        }
    }

}
