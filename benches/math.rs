use criterion::{black_box, criterion_group, criterion_main, Criterion};
use egg::{examples::math::rules, Runner};


fn integ_one_bench() {
    egg::test::test_runner(
        black_box("integ_one"),
        Some(Runner::default().with_node_limit(75_000)),
        black_box(&rules()),
        black_box(r#"
        (/ 1
           (- (/ (+ 1 (sqrt five))
                 2)
              (/ (- 1 (sqrt five))
                 2)))"#.parse().unwrap()),
        black_box(&["(/ 1 (sqrt five))".parse().unwrap()]),
        black_box(None),
        black_box(true)
    )
}



pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "integ_one",
        |b| b.iter(integ_one_bench)
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
