use egg::test;

mod schedulers;
use schedulers::schedulers::{IteratorScheduler, ParallelIteratorScheduler};

mod definitions;
use definitions::lambda;

use criterion::{criterion_group, criterion_main, Criterion};


const EXPRS: &'static [&'static str] = &[
    "(let zeroone (lam x
        (if (= (var x) 0)
            0
            1))
        (+ (app (var zeroone) 0)
        (app (var zeroone) 10)))",
    "(let compose (lam f (lam g (lam x (app (var f)
                                    (app (var g) (var x))))))
    (let repeat (fix repeat (lam fun (lam n
        (if (= (var n) 0)
            (lam i (var i))
            (app (app (var compose) (var fun))
                (app (app (var repeat)
                        (var fun))
                    (+ (var n) -1)))))))
    (let add1 (lam y (+ (var y) 1))
    (app (app (var repeat)
            (var add1))
        2))))",
    "(let fib (fix fib (lam n
        (if (= (var n) 0)
            0
        (if (= (var n) 1)
            1
        (+ (app (var fib)
                (+ (var n) -1))
            (app (var fib)
                (+ (var n) -2)))))))
        (app (var fib) 4))",
];

const EXTRA_PATTERNS: &'static [&'static str] = &[
    "(if (= (var ?x) ?e) ?then ?else)",
    "(+ (+ ?a ?b) ?c)",
    "(let ?v (fix ?v ?e) ?e)",
    "(app (lam ?v ?body) ?e)",
    "(let ?v ?e (app ?a ?b))",
    "(app (let ?v ?e ?a) (let ?v ?e ?b))",
    "(let ?v ?e (+   ?a ?b))",
    "(+   (let ?v ?e ?a) (let ?v ?e ?b))",
    "(let ?v ?e (=   ?a ?b))",
    "(=   (let ?v ?e ?a) (let ?v ?e ?b))",
    "(let ?v ?e (if ?cond ?then ?else))",
    "(if (let ?v ?e ?cond) (let ?v ?e ?then) (let ?v ?e ?else))",
    "(let ?v1 ?e (var ?v1))",
    "(let ?v1 ?e (var ?v2))",
    "(let ?v1 ?e (lam ?v1 ?body))",
    "(let ?v1 ?e (lam ?v2 ?body))",
    "(lam ?v2 (let ?v1 ?e ?body))",
    "(lam ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))",
];



pub fn lambda_ematching_benches_serial(c: &mut Criterion) {
    c.bench_function(
        "lambda_ematching_benches_serial",
        |b| b.iter(
            || test::bench_egraph(
                "lambda",
                lambda::rules(),
                EXPRS,
                EXTRA_PATTERNS,
                Some(egg::Runner::default().with_scheduler(IteratorScheduler))
            )
        )
    );
}

pub fn lambda_ematching_benches_parallel(c: &mut Criterion) {
    c.bench_function(
        "lambda_ematching_benches_parallel",
        |b| b.iter(
            || test::bench_egraph(
                "lambda",
                lambda::rules(),
                EXPRS,
                EXTRA_PATTERNS,
                Some(egg::Runner::default().with_scheduler(ParallelIteratorScheduler))
            )
        )
    );
}

pub fn lambda_ematching_benches_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambda_ematching_benches_comparison");
    group.bench_function(
        "serial",
        |b| b.iter(
            || test::bench_egraph(
                "lambda",
                lambda::rules(),
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
                "lambda",
                lambda::rules(),
                EXPRS,
                EXTRA_PATTERNS,
                Some(egg::Runner::default().with_scheduler(ParallelIteratorScheduler))
            )
        )
    );
    group.finish();

}

criterion_group!(
    benches,
    lambda_ematching_benches_comparison
);
criterion_main!(benches);
