use criterion::{criterion_group, criterion_main, Criterion};
use egg::{examples::math::rules, Runner};


fn math_simplify_root() {
    egg::test::test_runner(
        "math_simplify_root",
        Some(Runner::default().with_node_limit(75_000)),
        &rules(),
        r#"
        (/ 1
           (- (/ (+ 1 (sqrt five))
                 2)
              (/ (- 1 (sqrt five))
                 2)))"#.parse().unwrap(),
        &["(/ 1 (sqrt five))".parse().unwrap()],
        None,
        true
    )
}

fn math_simplify_factor() {
    egg::test::test_runner(
        "math_simplify_factor",
        None,
        &rules(),
        "(* (+ x 3) (+ x 1))".parse().unwrap(),
        &["(+ (+ (* x x) (* 4 x)) 3)".parse().unwrap()],
        None,
        true
    )
}

pub fn math_bench(c: &mut Criterion) {
    c.bench_function(
        "math_simplify_root",
        |b| b.iter(math_simplify_root)
    );
    c.bench_function(
        "math_simplify_factor",
        |b| b.iter(math_simplify_factor)
    );
}

criterion_group!(benches, math_bench);
criterion_main!(benches);
