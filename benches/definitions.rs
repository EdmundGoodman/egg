#[allow(dead_code)]
pub mod simple {
    use egg::*;

    define_language! {
        pub enum SimpleLanguage {
            Num(i32),
            "+" = Add([Id; 2]),
            "*" = Mul([Id; 2]),
            Symbol(Symbol),
        }
    }

    pub fn make_rules() -> Vec<Rewrite<SimpleLanguage, ()>> {
        vec![
            rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
            rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
            rewrite!("add-0"; "(+ ?a 0)" => "?a"),
            rewrite!("mul-0"; "(* ?a 0)" => "0"),
            rewrite!("mul-1"; "(* ?a 1)" => "?a"),
        ]
    }

    // pub const EXAMPLE_INPUTS: &'static [&'static str] = &[
    //     "(* 0 42)",
    //     "(+ 0 (* 1 foo))"
    // ];
}

#[allow(dead_code)]
pub mod math {
    use egg::{rewrite as rw, *};
    use ordered_float::NotNan;

    pub type EGraph = egg::EGraph<Math, ConstantFold>;
    pub type Rewrite = egg::Rewrite<Math, ConstantFold>;

    pub type Constant = NotNan<f64>;

    define_language! {
        pub enum Math {
            "d" = Diff([Id; 2]),
            "i" = Integral([Id; 2]),

            "+" = Add([Id; 2]),
            "-" = Sub([Id; 2]),
            "*" = Mul([Id; 2]),
            "/" = Div([Id; 2]),
            "pow" = Pow([Id; 2]),
            "ln" = Ln(Id),
            "sqrt" = Sqrt(Id),

            "sin" = Sin(Id),
            "cos" = Cos(Id),

            Constant(Constant),
            Symbol(Symbol),
        }
    }

    // You could use egg::AstSize, but this is useful for debugging, since
    // it will really try to get rid of the Diff operator
    pub struct MathCostFn;
    impl egg::CostFunction<Math> for MathCostFn {
        type Cost = usize;
        fn cost<C>(&mut self, enode: &Math, mut costs: C) -> Self::Cost
        where
            C: FnMut(Id) -> Self::Cost,
        {
            let op_cost = match enode {
                Math::Diff(..) => 100,
                Math::Integral(..) => 100,
                _ => 1,
            };
            enode.fold(op_cost, |sum, i| sum + costs(i))
        }
    }

    #[derive(Default)]
    pub struct ConstantFold;
    impl Analysis<Math> for ConstantFold {
        type Data = Option<(Constant, PatternAst<Math>)>;

        fn make(egraph: &mut EGraph, enode: &Math) -> Self::Data {
            let x = |i: &Id| egraph[*i].data.as_ref().map(|d| d.0);
            Some(match enode {
                Math::Constant(c) => (*c, format!("{}", c).parse().unwrap()),
                Math::Add([a, b]) => (
                    x(a)? + x(b)?,
                    format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
                ),
                Math::Sub([a, b]) => (
                    x(a)? - x(b)?,
                    format!("(- {} {})", x(a)?, x(b)?).parse().unwrap(),
                ),
                Math::Mul([a, b]) => (
                    x(a)? * x(b)?,
                    format!("(* {} {})", x(a)?, x(b)?).parse().unwrap(),
                ),
                Math::Div([a, b]) if x(b) != Some(NotNan::new(0.0).unwrap()) => (
                    x(a)? / x(b)?,
                    format!("(/ {} {})", x(a)?, x(b)?).parse().unwrap(),
                ),
                _ => return None,
            })
        }

        fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
            merge_option(to, from, |a, b| {
                assert_eq!(a.0, b.0, "Merged non-equal constants");
                DidMerge(false, false)
            })
        }

        fn modify(egraph: &mut EGraph, id: Id) {
            let data = egraph[id].data.clone();
            if let Some((c, pat)) = data {
                if egraph.are_explanations_enabled() {
                    egraph.union_instantiations(
                        &pat,
                        &format!("{}", c).parse().unwrap(),
                        &Default::default(),
                        "constant_fold".to_string(),
                    );
                } else {
                    let added = egraph.add(Math::Constant(c));
                    egraph.union(id, added);
                }
                // to not prune, comment this out
                egraph[id].nodes.retain(|n| n.is_leaf());

                #[cfg(debug_assertions)]
                egraph[id].assert_unique_leaves();
            }
        }
    }

    fn is_const_or_distinct_var(v: &str, w: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let v = v.parse().unwrap();
        let w = w.parse().unwrap();
        move |egraph, _, subst| {
            egraph.find(subst[v]) != egraph.find(subst[w])
                && (egraph[subst[v]].data.is_some()
                    || egraph[subst[v]]
                        .nodes
                        .iter()
                        .any(|n| matches!(n, Math::Symbol(..))))
        }
    }

    fn is_const(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| egraph[subst[var]].data.is_some()
    }

    fn is_sym(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| {
            egraph[subst[var]]
                .nodes
                .iter()
                .any(|n| matches!(n, Math::Symbol(..)))
        }
    }

    fn is_not_zero(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| {
            if let Some(n) = &egraph[subst[var]].data {
                *(n.0) != 0.0
            } else {
                true
            }
        }
    }

    #[rustfmt::skip]
    pub fn rules() -> Vec<Rewrite> { vec![
        rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),

        rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        rw!("div-canon"; "(/ ?a ?b)" => "(* ?a (pow ?b -1))" if is_not_zero("?b")),
        // rw!("canon-sub"; "(+ ?a (* -1 ?b))"   => "(- ?a ?b)"),
        // rw!("canon-div"; "(* ?a (pow ?b -1))" => "(/ ?a ?b)" if is_not_zero("?b")),

        rw!("zero-add"; "(+ ?a 0)" => "?a"),
        rw!("zero-mul"; "(* ?a 0)" => "0"),
        rw!("one-mul";  "(* ?a 1)" => "?a"),

        rw!("add-zero"; "?a" => "(+ ?a 0)"),
        rw!("mul-one";  "?a" => "(* ?a 1)"),

        rw!("cancel-sub"; "(- ?a ?a)" => "0"),
        rw!("cancel-div"; "(/ ?a ?a)" => "1" if is_not_zero("?a")),

        rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),

        rw!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
        rw!("pow0"; "(pow ?x 0)" => "1"
            if is_not_zero("?x")),
        rw!("pow1"; "(pow ?x 1)" => "?x"),
        rw!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
        rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
            if is_not_zero("?x")),
        rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero("?x")),

        rw!("d-variable"; "(d ?x ?x)" => "1" if is_sym("?x")),
        rw!("d-constant"; "(d ?x ?c)" => "0" if is_sym("?x") if is_const_or_distinct_var("?c", "?x")),

        rw!("d-add"; "(d ?x (+ ?a ?b))" => "(+ (d ?x ?a) (d ?x ?b))"),
        rw!("d-mul"; "(d ?x (* ?a ?b))" => "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))"),

        rw!("d-sin"; "(d ?x (sin ?x))" => "(cos ?x)"),
        rw!("d-cos"; "(d ?x (cos ?x))" => "(* -1 (sin ?x))"),

        rw!("d-ln"; "(d ?x (ln ?x))" => "(/ 1 ?x)" if is_not_zero("?x")),

        rw!("d-power";
            "(d ?x (pow ?f ?g))" =>
            "(* (pow ?f ?g)
                (+ (* (d ?x ?f)
                      (/ ?g ?f))
                   (* (d ?x ?g)
                      (ln ?f))))"
            if is_not_zero("?f")
            if is_not_zero("?g")
        ),

        rw!("i-one"; "(i 1 ?x)" => "?x"),
        rw!("i-power-const"; "(i (pow ?x ?c) ?x)" =>
            "(/ (pow ?x (+ ?c 1)) (+ ?c 1))" if is_const("?c")),
        rw!("i-cos"; "(i (cos ?x) ?x)" => "(sin ?x)"),
        rw!("i-sin"; "(i (sin ?x) ?x)" => "(* -1 (cos ?x))"),
        rw!("i-sum"; "(i (+ ?f ?g) ?x)" => "(+ (i ?f ?x) (i ?g ?x))"),
        rw!("i-dif"; "(i (- ?f ?g) ?x)" => "(- (i ?f ?x) (i ?g ?x))"),
        rw!("i-parts"; "(i (* ?a ?b) ?x)" =>
            "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))"),
    ]}
}

#[allow(dead_code)]
pub mod lambda {
    use egg::{rewrite as rw, *};
    use rustc_hash::FxHashSet as HashSet;

    define_language! {
        pub enum Lambda {
            Bool(bool),
            Num(i32),

            "var" = Var(Id),

            "+" = Add([Id; 2]),
            "=" = Eq([Id; 2]),

            "app" = App([Id; 2]),
            "lam" = Lambda([Id; 2]),
            "let" = Let([Id; 3]),
            "fix" = Fix([Id; 2]),

            "if" = If([Id; 3]),

            Symbol(egg::Symbol),
        }
    }

    impl Lambda {
        fn num(&self) -> Option<i32> {
            match self {
                Lambda::Num(n) => Some(*n),
                _ => None,
            }
        }
    }

    type EGraph = egg::EGraph<Lambda, LambdaAnalysis>;

    #[derive(Default)]
    pub struct LambdaAnalysis;

    #[derive(Debug)]
    pub struct Data {
        free: HashSet<Id>,
        constant: Option<(Lambda, PatternAst<Lambda>)>,
    }

    pub fn eval(egraph: &EGraph, enode: &Lambda) -> Option<(Lambda, PatternAst<Lambda>)> {
        let x = |i: &Id| egraph[*i].data.constant.as_ref().map(|c| &c.0);
        match enode {
            Lambda::Num(n) => Some((enode.clone(), format!("{}", n).parse().unwrap())),
            Lambda::Bool(b) => Some((enode.clone(), format!("{}", b).parse().unwrap())),
            Lambda::Add([a, b]) => Some((
                Lambda::Num(x(a)?.num()?.checked_add(x(b)?.num()?)?),
                format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
            )),
            Lambda::Eq([a, b]) => Some((
                Lambda::Bool(x(a)? == x(b)?),
                format!("(= {} {})", x(a)?, x(b)?).parse().unwrap(),
            )),
            _ => None,
        }
    }

    impl Analysis<Lambda> for LambdaAnalysis {
        type Data = Data;
        fn merge(&mut self, to: &mut Data, from: Data) -> DidMerge {
            let before_len = to.free.len();
            // to.free.extend(from.free);
            to.free.retain(|i| from.free.contains(i));
            // compare lengths to see if I changed to or from
            DidMerge(
                before_len != to.free.len(),
                to.free.len() != from.free.len(),
            ) | merge_option(&mut to.constant, from.constant, |a, b| {
                assert_eq!(a.0, b.0, "Merged non-equal constants");
                DidMerge(false, false)
            })
        }

        fn make(egraph: &mut EGraph, enode: &Lambda) -> Data {
            let f = |i: &Id| egraph[*i].data.free.iter().cloned();
            let mut free = HashSet::default();
            match enode {
                Lambda::Var(v) => {
                    free.insert(*v);
                }
                Lambda::Let([v, a, b]) => {
                    free.extend(f(b));
                    free.remove(v);
                    free.extend(f(a));
                }
                Lambda::Lambda([v, a]) | Lambda::Fix([v, a]) => {
                    free.extend(f(a));
                    free.remove(v);
                }
                _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
            }
            let constant = eval(egraph, enode);
            Data { constant, free }
        }

        fn modify(egraph: &mut EGraph, id: Id) {
            if let Some(c) = egraph[id].data.constant.clone() {
                if egraph.are_explanations_enabled() {
                    egraph.union_instantiations(
                        &c.0.to_string().parse().unwrap(),
                        &c.1,
                        &Default::default(),
                        "analysis".to_string(),
                    );
                } else {
                    let const_id = egraph.add(c.0);
                    egraph.union(id, const_id);
                }
            }
        }
    }

    pub fn var(s: &str) -> Var {
        s.parse().unwrap()
    }

    pub fn is_not_same_var(v1: Var, v2: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        move |egraph, _, subst| egraph.find(subst[v1]) != egraph.find(subst[v2])
    }

    pub fn is_const(v: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        move |egraph, _, subst| egraph[subst[v]].data.constant.is_some()
    }

    pub fn rules() -> Vec<Rewrite<Lambda, LambdaAnalysis>> {
        vec![
            // open term rules
            rw!("if-true";  "(if  true ?then ?else)" => "?then"),
            rw!("if-false"; "(if false ?then ?else)" => "?else"),
            rw!("if-elim"; "(if (= (var ?x) ?e) ?then ?else)" => "?else"
                if ConditionEqual::parse("(let ?x ?e ?then)", "(let ?x ?e ?else)")),
            rw!("add-comm";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
            rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),
            rw!("eq-comm";   "(= ?a ?b)"        => "(= ?b ?a)"),
            // subst rules
            rw!("fix";      "(fix ?v ?e)"             => "(let ?v (fix ?v ?e) ?e)"),
            rw!("beta";     "(app (lam ?v ?body) ?e)" => "(let ?v ?e ?body)"),
            rw!("let-app";  "(let ?v ?e (app ?a ?b))" => "(app (let ?v ?e ?a) (let ?v ?e ?b))"),
            rw!("let-add";  "(let ?v ?e (+   ?a ?b))" => "(+   (let ?v ?e ?a) (let ?v ?e ?b))"),
            rw!("let-eq";   "(let ?v ?e (=   ?a ?b))" => "(=   (let ?v ?e ?a) (let ?v ?e ?b))"),
            rw!("let-const";
                "(let ?v ?e ?c)" => "?c" if is_const(var("?c"))),
            rw!("let-if";
                "(let ?v ?e (if ?cond ?then ?else))" =>
                "(if (let ?v ?e ?cond) (let ?v ?e ?then) (let ?v ?e ?else))"
            ),
            rw!("let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
            rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" => "(var ?v2)"
                if is_not_same_var(var("?v1"), var("?v2"))),
            rw!("let-lam-same"; "(let ?v1 ?e (lam ?v1 ?body))" => "(lam ?v1 ?body)"),
            rw!("let-lam-diff";
                "(let ?v1 ?e (lam ?v2 ?body))" =>
                { CaptureAvoid {
                    fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
                    if_not_free: "(lam ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
                    if_free: "(lam ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
                }}
                if is_not_same_var(var("?v1"), var("?v2"))),
        ]
    }

    pub struct CaptureAvoid {
        fresh: Var,
        v2: Var,
        e: Var,
        if_not_free: Pattern<Lambda>,
        if_free: Pattern<Lambda>,
    }

    impl Applier<Lambda, LambdaAnalysis> for CaptureAvoid {
        fn apply_one(
            &self,
            egraph: &mut EGraph,
            eclass: Id,
            subst: &Subst,
            searcher_ast: Option<&PatternAst<Lambda>>,
            rule_name: Symbol,
        ) -> Vec<Id> {
            let e = subst[self.e];
            let v2 = subst[self.v2];
            let v2_free_in_e = egraph[e].data.free.contains(&v2);
            if v2_free_in_e {
                let mut subst = subst.clone();
                let sym = Lambda::Symbol(format!("_{}", eclass).into());
                subst.insert(self.fresh, egraph.add(sym));
                self.if_free
                    .apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
            } else {
                self.if_not_free
                    .apply_one(egraph, eclass, subst, searcher_ast, rule_name)
            }
        }
    }

}
