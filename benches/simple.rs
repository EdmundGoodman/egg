use egg::*;

mod definitions;
use definitions::simple;

use criterion::{criterion_group, criterion_main, Criterion};


fn simplify(s: &str) -> String {
    let expr: RecExpr<simple::SimpleLanguage> = s.parse().unwrap();
    let runner = Runner::default()
        .with_scheduler(SimpleScheduler)
        .with_expr(&expr)
        .run(&simple::make_rules());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_best_cost, best) = extractor.find_best(root);
    best.to_string()
}

pub fn simple_bench(c: &mut Criterion) {
    c.bench_function(
        "simplify",
        |b| b.iter(
            || simplify("(+ 0 (* 1 foo))")
        )
    );
}


criterion_group!(benches, simple_bench);
criterion_main!(benches);
