use egg::{rewrite as rw, *};
use egg::examples::math::{Math, ConstantFold, rules};


egg::test_fn! {
    math_associate_adds, [
        rw!("comm-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
    ],
    runner = Runner::default()
        .with_iter_limit(7)
        .with_scheduler(SimpleScheduler),
    "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 7))))))"
    =>
    "(+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 1))))))"
    @check |r: Runner<Math, ()>| assert_eq!(r.egraph.number_of_classes(), 127)
}

egg::test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    math_fail, rules(),
    "(+ x y)" => "(/ x y)"
}

egg::test_fn! {math_simplify_add, rules(), "(+ x (+ x (+ x x)))" => "(* 4 x)" }
egg::test_fn! {math_powers, rules(), "(* (pow 2 x) (pow 2 y))" => "(pow 2 (+ x y))"}

egg::test_fn! {
    math_simplify_const, rules(),
    "(+ 1 (- a (* (- 2 1) a)))" => "1"
}

egg::test_fn! {
    math_simplify_root, rules(),
    runner = Runner::default().with_node_limit(75_000),
    r#"
    (/ 1
       (- (/ (+ 1 (sqrt five))
             2)
          (/ (- 1 (sqrt five))
             2)))"#
    =>
    "(/ 1 (sqrt five))"
}

egg::test_fn! {
    math_simplify_factor, rules(),
    "(* (+ x 3) (+ x 1))"
    =>
    "(+ (+ (* x x) (* 4 x)) 3)"
}

egg::test_fn! {math_diff_same,      rules(), "(d x x)" => "1"}
egg::test_fn! {math_diff_different, rules(), "(d x y)" => "0"}
egg::test_fn! {math_diff_simple1,   rules(), "(d x (+ 1 (* 2 x)))" => "2"}
egg::test_fn! {math_diff_simple2,   rules(), "(d x (+ 1 (* y x)))" => "y"}
egg::test_fn! {math_diff_ln,        rules(), "(d x (ln x))" => "(/ 1 x)"}

egg::test_fn! {
    diff_power_simple, rules(),
    "(d x (pow x 3))" => "(* 3 (pow x 2))"
}

egg::test_fn! {
    diff_power_harder, rules(),
    runner = Runner::default()
        .with_time_limit(std::time::Duration::from_secs(10))
        .with_iter_limit(60)
        .with_node_limit(100_000)
        .with_explanations_enabled()
        // HACK this needs to "see" the end expression
        .with_expr(&"(* x (- (* 3 x) 14))".parse().unwrap()),
    "(d x (- (pow x 3) (* 7 (pow x 2))))"
    =>
    "(* x (- (* 3 x) 14))"
}

egg::test_fn! {
    integ_one, rules(), "(i 1 x)" => "x"
}

egg::test_fn! {
    integ_sin, rules(), "(i (cos x) x)" => "(sin x)"
}

egg::test_fn! {
    integ_x, rules(), "(i (pow x 1) x)" => "(/ (pow x 2) 2)"
}

egg::test_fn! {
    integ_part1, rules(), "(i (* x (cos x)) x)" => "(+ (* x (sin x)) (cos x))"
}

egg::test_fn! {
    integ_part2, rules(),
    "(i (* (cos x) x) x)" => "(+ (* x (sin x)) (cos x))"
}

egg::test_fn! {
    integ_part3, rules(), "(i (ln x) x)" => "(- (* x (ln x)) x)"
}

#[test]
fn assoc_mul_saturates() {
    let expr: RecExpr<Math> = "(* x 1)".parse().unwrap();

    let runner: Runner<Math, ConstantFold> = Runner::default()
        .with_iter_limit(3)
        .with_expr(&expr)
        .run(&rules());

    assert!(matches!(runner.stop_reason, Some(StopReason::Saturated)));
}

#[test]
fn test_union_trusted() {
    let expr: RecExpr<Math> = "(+ (* x 1) y)".parse().unwrap();
    let expr2 = "20".parse().unwrap();
    let mut runner: Runner<Math, ConstantFold> = Runner::default()
        .with_explanations_enabled()
        .with_iter_limit(3)
        .with_expr(&expr)
        .run(&rules());
    let lhs = runner.egraph.add_expr(&expr);
    let rhs = runner.egraph.add_expr(&expr2);
    runner.egraph.union_trusted(lhs, rhs, "whatever");
    let proof = runner.explain_equivalence(&expr, &expr2).get_flat_strings();
    assert_eq!(proof, vec!["(+ (* x 1) y)", "(Rewrite=> whatever 20)"]);
}

#[cfg(feature = "lp")]
#[test]
fn math_lp_extract() {
    let expr: RecExpr<Math> = "(pow (+ x (+ x x)) (+ x x))".parse().unwrap();

    let runner: Runner<Math, ConstantFold> = Runner::default()
        .with_iter_limit(3)
        .with_expr(&expr)
        .run(&rules());
    let root = runner.roots[0];

    let best = Extractor::new(&runner.egraph, AstSize).find_best(root).1;
    let lp_best = LpExtractor::new(&runner.egraph, AstSize).solve(root);

    println!("input   [{}] {}", expr.len(), expr);
    println!("normal  [{}] {}", best.len(), best);
    println!("ilp cse [{}] {}", lp_best.len(), lp_best);

    assert_ne!(best, lp_best);
    assert_eq!(lp_best.len(), 4);
}

#[test]
fn math_ematching_bench() {
    let exprs = &[
        "(i (ln x) x)",
        "(i (+ x (cos x)) x)",
        "(i (* (cos x) x) x)",
        "(d x (+ 1 (* 2 x)))",
        "(d x (- (pow x 3) (* 7 (pow x 2))))",
        "(+ (* y (+ x y)) (- (+ x 2) (+ x x)))",
        "(/ 1 (- (/ (+ 1 (sqrt five)) 2) (/ (- 1 (sqrt five)) 2)))",
    ];

    let extra_patterns = &[
        "(+ ?a (+ ?b ?c))",
        "(+ (+ ?a ?b) ?c)",
        "(* ?a (* ?b ?c))",
        "(* (* ?a ?b) ?c)",
        "(+ ?a (* -1 ?b))",
        "(* ?a (pow ?b -1))",
        "(* ?a (+ ?b ?c))",
        "(pow ?a (+ ?b ?c))",
        "(+ (* ?a ?b) (* ?a ?c))",
        "(* (pow ?a ?b) (pow ?a ?c))",
        "(* ?x (/ 1 ?x))",
        "(d ?x (+ ?a ?b))",
        "(+ (d ?x ?a) (d ?x ?b))",
        "(d ?x (* ?a ?b))",
        "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))",
        "(d ?x (sin ?x))",
        "(d ?x (cos ?x))",
        "(* -1 (sin ?x))",
        "(* -1 (cos ?x))",
        "(i (cos ?x) ?x)",
        "(i (sin ?x) ?x)",
        "(d ?x (ln ?x))",
        "(d ?x (pow ?f ?g))",
        "(* (pow ?f ?g) (+ (* (d ?x ?f) (/ ?g ?f)) (* (d ?x ?g) (ln ?f))))",
        "(i (pow ?x ?c) ?x)",
        "(/ (pow ?x (+ ?c 1)) (+ ?c 1))",
        "(i (+ ?f ?g) ?x)",
        "(i (- ?f ?g) ?x)",
        "(+ (i ?f ?x) (i ?g ?x))",
        "(- (i ?f ?x) (i ?g ?x))",
        "(i (* ?a ?b) ?x)",
        "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))",
    ];

    egg::test::bench_egraph("math", rules(), exprs, extra_patterns);
}

#[test]
fn test_basic_egraph_union_intersect() {
    let mut egraph1 = EGraph::new(ConstantFold {}).with_explanations_enabled();
    let mut egraph2 = EGraph::new(ConstantFold {}).with_explanations_enabled();
    egraph1.union_instantiations(
        &"x".parse().unwrap(),
        &"y".parse().unwrap(),
        &Default::default(),
        "",
    );
    egraph1.union_instantiations(
        &"y".parse().unwrap(),
        &"z".parse().unwrap(),
        &Default::default(),
        "",
    );
    egraph2.union_instantiations(
        &"x".parse().unwrap(),
        &"y".parse().unwrap(),
        &Default::default(),
        "",
    );
    egraph2.union_instantiations(
        &"x".parse().unwrap(),
        &"a".parse().unwrap(),
        &Default::default(),
        "",
    );

    let mut egraph3 = egraph1.egraph_intersect(&egraph2, ConstantFold {});

    egraph2.egraph_union(&egraph1);

    assert_eq!(
        egraph2.add_expr(&"x".parse().unwrap()),
        egraph2.add_expr(&"y".parse().unwrap())
    );
    assert_eq!(
        egraph3.add_expr(&"x".parse().unwrap()),
        egraph3.add_expr(&"y".parse().unwrap())
    );

    assert_eq!(
        egraph2.add_expr(&"x".parse().unwrap()),
        egraph2.add_expr(&"z".parse().unwrap())
    );
    assert_ne!(
        egraph3.add_expr(&"x".parse().unwrap()),
        egraph3.add_expr(&"z".parse().unwrap())
    );
    assert_eq!(
        egraph2.add_expr(&"x".parse().unwrap()),
        egraph2.add_expr(&"a".parse().unwrap())
    );
    assert_ne!(
        egraph3.add_expr(&"x".parse().unwrap()),
        egraph3.add_expr(&"a".parse().unwrap())
    );

    assert_eq!(
        egraph2.add_expr(&"y".parse().unwrap()),
        egraph2.add_expr(&"a".parse().unwrap())
    );
    assert_ne!(
        egraph3.add_expr(&"y".parse().unwrap()),
        egraph3.add_expr(&"a".parse().unwrap())
    );
}

#[test]
fn test_intersect_basic() {
    let mut egraph1 = EGraph::new(ConstantFold {}).with_explanations_enabled();
    let mut egraph2 = EGraph::new(ConstantFold {}).with_explanations_enabled();
    egraph1.union_instantiations(
        &"(+ x 0)".parse().unwrap(),
        &"(+ y 0)".parse().unwrap(),
        &Default::default(),
        "",
    );
    egraph2.union_instantiations(
        &"x".parse().unwrap(),
        &"y".parse().unwrap(),
        &Default::default(),
        "",
    );
    egraph2.add_expr(&"(+ x 0)".parse().unwrap());
    egraph2.add_expr(&"(+ y 0)".parse().unwrap());

    let mut egraph3 = egraph1.egraph_intersect(&egraph2, ConstantFold {});

    assert_ne!(
        egraph3.add_expr(&"x".parse().unwrap()),
        egraph3.add_expr(&"y".parse().unwrap())
    );
    assert_eq!(
        egraph3.add_expr(&"(+ x 0)".parse().unwrap()),
        egraph3.add_expr(&"(+ y 0)".parse().unwrap())
    );
}

#[test]
fn test_medium_intersect() {
    let mut egraph1 = egg::EGraph::<Math, ()>::new(());

    egraph1.add_expr(&"(sqrt (ln 1))".parse().unwrap());
    let ln = egraph1.add_expr(&"(ln 1)".parse().unwrap());
    let a = egraph1.add_expr(&"(sqrt (sin pi))".parse().unwrap());
    let b = egraph1.add_expr(&"(* 1 pi)".parse().unwrap());
    let pi = egraph1.add_expr(&"pi".parse().unwrap());
    egraph1.union(a, b);
    egraph1.union(a, pi);
    let c = egraph1.add_expr(&"(+ pi pi)".parse().unwrap());
    egraph1.union(ln, c);
    let k = egraph1.add_expr(&"k".parse().unwrap());
    let one = egraph1.add_expr(&"1".parse().unwrap());
    egraph1.union(k, one);
    egraph1.rebuild();

    assert_eq!(
        egraph1.add_expr(&"(ln k)".parse().unwrap()),
        egraph1.add_expr(&"(+ (* k pi) (* k pi))".parse().unwrap())
    );

    let mut egraph2 = egg::EGraph::<Math, ()>::new(());
    let ln = egraph2.add_expr(&"(ln 2)".parse().unwrap());
    let k = egraph2.add_expr(&"k".parse().unwrap());
    let mk1 = egraph2.add_expr(&"(* k 1)".parse().unwrap());
    egraph2.union(mk1, k);
    let two = egraph2.add_expr(&"2".parse().unwrap());
    egraph2.union(mk1, two);
    let mul2pi = egraph2.add_expr(&"(+ (* 2 pi) (* 2 pi))".parse().unwrap());
    egraph2.union(ln, mul2pi);
    egraph2.rebuild();

    assert_eq!(
        egraph2.add_expr(&"(ln k)".parse().unwrap()),
        egraph2.add_expr(&"(+ (* k pi) (* k pi))".parse().unwrap())
    );

    let mut egraph3 = egraph1.egraph_intersect(&egraph2, ());

    assert_eq!(
        egraph3.add_expr(&"(ln k)".parse().unwrap()),
        egraph3.add_expr(&"(+ (* k pi) (* k pi))".parse().unwrap())
    );
}
