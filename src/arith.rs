
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Term {
    True,
    False,
    If(Box<Term>, Box<Term>, Box<Term>),
    Zero,
    Succ(Box<Term>),
    Pred(Box<Term>),
    IsZero(Box<Term>)
}

fn is_numric_value(t: &Term) -> bool {
    match *t {
        Term::Zero => true,
        Term::Succ(ref t) => is_numric_value(t),
        _ => false
    }
}

fn eval_1step(t: Term) -> Result<Term, &'static str> {
    use self::Term::*;
    match t {
        If(box True, box t, _) => Ok(t),
        If(box False, _, box t) => Ok(t),
        If(box t1, t2, t3) => {
            let t1 = box eval_1step(t1)?;
            Ok(If(t1, t2, t3))
        },
        Succ(box t) => {
            let t = box eval_1step(t)?;
            Ok(Succ(t))
        },
        Pred(box Zero) => Ok(Zero),
        Pred(box Succ(box ref t)) if is_numric_value(t) => Ok(t.clone()),
        Pred(box t) => {
            let t = box eval_1step(t)?;
            Ok(Pred(t))
        },
        IsZero(box Zero) => Ok(True),
        IsZero(box Succ(ref t)) if is_numric_value(t) => Ok(False),
        IsZero(box t) => {
            let t = box eval_1step(t)?;
            Ok(IsZero(t))
        },
        _ => Err("Wrong")
    }
}

pub fn eval(t: Term) -> Term {
    match eval_1step(t.clone()) {
        Ok(t) => eval(t),
        Err(_) => t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Term::*;

    #[test]
    fn test_eval_1step() {
        assert_eq! {
            eval_1step(If(box True, box Zero, box False)).unwrap(),
            Zero
        }
        assert_eq! {
            eval_1step(If(box False, box Zero, box Succ(box Zero))).unwrap(),
            Succ(box Zero)
        }
        assert_eq! {
            eval_1step(Pred(box Succ(box Succ(box Zero)))).unwrap(),
            Succ(box Zero)
        }
        assert_eq! {
            eval_1step(If(box IsZero(box Pred(box Pred(box Succ(box Zero)))), box True, box False)).unwrap(),
            If(box IsZero(box Pred(box Zero)), box True, box False)
        }
    }

    #[test]
    fn test_eval() {
        assert_eq! {
            eval(If(box IsZero(box Pred(box Pred(box Succ(box Zero)))),
                box If(box True, box Succ(box Zero), box False),
                box False
            )),
            Succ(box Zero)
        }
        assert_eq!(eval(Succ(box Zero)), Succ(box Zero));
    }
}
