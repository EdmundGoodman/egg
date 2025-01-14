use egg::*;

mod definitions;
use definitions::simple;


use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};


fn serial_simplify(s: &str) -> String {
    let expr: RecExpr<simple::SimpleLanguage> = s.parse().unwrap();
    let runner = Runner::default()
        .with_scheduler(IteratorScheduler)
        .with_expr(&expr)
        .run(&simple::make_rules());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_best_cost, best) = extractor.find_best(root);
    best.to_string()
}

fn parallel_simplify(s: &str) -> String {
    let expr: RecExpr<simple::SimpleLanguage> = s.parse().unwrap();
    let runner = Runner::default()
        .with_scheduler(ParallelIteratorScheduler)
        .with_expr(&expr)
        .run(&simple::make_rules());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_best_cost, best) = extractor.find_best(root);
    best.to_string()
}

pub fn serial_simple_bench(c: &mut Criterion) {
    c.bench_function(
        "serial_simplify",
        |b| b.iter(
            || serial_simplify("(+ 0 (* 1 foo))")
        )
    );
}

pub fn parallel_simple_bench(c: &mut Criterion) {
    c.bench_function(
        "parallel_simplify",
        |b| b.iter(
            || parallel_simplify("(+ 0 (* 1 foo))")
        )
    );
}

pub fn comparison_simple_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("simplify");
    for i in simple::EXAMPLE_INPUTS.iter() {
        group.bench_with_input(BenchmarkId::new("serial_simplify", i), i,
            |b, i| b.iter(|| serial_simplify(*i)));
        group.bench_with_input(BenchmarkId::new("parallel_simplify", i), i,
            |b, i| b.iter(|| parallel_simplify(*i)));
    }
    group.finish();
}


criterion_group!(benches, comparison_simple_bench);
criterion_main!(benches);
