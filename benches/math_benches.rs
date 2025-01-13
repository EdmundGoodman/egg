use egg::{test, IteratorScheduler, ParallelIteratorScheduler};

mod definitions;
use definitions::math;

use criterion::{criterion_group, criterion_main, Criterion};


const EXPRS: &'static [&'static str] = &[
    "(i (ln x) x)",
    "(i (+ x (cos x)) x)",
    "(i (* (cos x) x) x)",
    "(d x (+ 1 (* 2 x)))",
    "(d x (- (pow x 3) (* 7 (pow x 2))))",
    "(+ (* y (+ x y)) (- (+ x 2) (+ x x)))",
    "(/ 1 (- (/ (+ 1 (sqrt five)) 2) (/ (- 1 (sqrt five)) 2)))",
];

const EXTRA_PATTERNS: &'static [&'static str] = &[
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



pub fn ematching_benches_serial(c: &mut Criterion) {
    c.bench_function(
        "ematching_benches",
        |b| b.iter(
            || test::bench_egraph(
                "math",
                math::rules(),
                EXPRS,
                EXTRA_PATTERNS,
                Some(egg::Runner::default().with_scheduler(IteratorScheduler))
            )
        )
    );
}

pub fn ematching_benches_parallel(c: &mut Criterion) {
    c.bench_function(
        "ematching_benches_parallel",
        |b| b.iter(
            || test::bench_egraph(
                "math",
                math::rules(),
                EXPRS,
                EXTRA_PATTERNS,
                Some(egg::Runner::default().with_scheduler(ParallelIteratorScheduler))
            )
        )
    );
}

pub fn ematching_benches_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("ematching_benches_comparison");
    group.bench_function(
        "serial",
        |b| b.iter(
            || test::bench_egraph(
                "math",
                math::rules(),
                EXPRS,
                EXTRA_PATTERNS,
                Some(egg::Runner::default().with_scheduler(IteratorScheduler))
            )
        )
    );
    group.bench_function(
        "parallel",
        |b| b.iter(
            || test::bench_egraph(
                "math",
                math::rules(),
                EXPRS,
                EXTRA_PATTERNS,
                Some(egg::Runner::default().with_scheduler(ParallelIteratorScheduler))
            )
        )
    );
    group.finish();

}

criterion_group!(benches, ematching_benches_comparison);
criterion_main!(benches);
