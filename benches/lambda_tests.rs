use egg::*;

mod schedulers;
use schedulers::schedulers::{IteratorScheduler, ParallelIteratorScheduler};

mod definitions;
use definitions::lambda;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};


fn lambda_under() {
    egg::test::test_runner(
        "lambda_under",
        None,
        &lambda::rules(),
        "(lam x (+ 4
        (app (lam y (var y))
             4)))".parse().unwrap(),
        // "(lam x (+ 4 (let y 4 (var y))))",
        // "(lam x (+ 4 4))",
        &["(lam x 8))".parse().unwrap()],
        None,
        true
    );
}

fn lambda_if_elim() {
    egg::test::test_runner(
        "lambda_if_elim",
        None,
        &lambda::rules(),
        "(if (= (var a) (var b))
         (+ (var a) (var a))
         (+ (var a) (var b)))".parse().unwrap(),
        &["(+ (var a) (var b))".parse().unwrap()],
        None,
        true
    );
}

fn lambda_let_simple() {
    egg::test::test_runner(
        "lambda_let_simple",
        None,
        &lambda::rules(),
        "(let x 0
     (let y 1
     (+ (var x) (var y))))".parse().unwrap(),
        // "(let ?a 0
        //  (+ (var ?a) 1))",
        // "(+ 0 1)",
        &["1".parse().unwrap()],
        None,
        true
    );
}

fn lambda_compose() {
    egg::test::test_runner(
        "lambda_compose",
        None,
        &lambda::rules(),
        "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1)) (var add1))))".parse().unwrap(),
        &[
            "(lam ?x (+ 1
                (app (lam ?y (+ 1 (var ?y)))
                     (var ?x))))".parse().unwrap(),
                "(lam ?x (+ (var ?x) 2))".parse().unwrap()
        ],
        None,
        true
    );
}

fn lambda_if_simple() {
    egg::test::test_runner(
        "lambda_if_simple",
        None,
        &lambda::rules(),
        "(if (= 1 1) 7 9)".parse().unwrap(),
        &["7".parse().unwrap()],
        None,
        true
    );
}

fn lambda_compose_many() {
    egg::test::test_runner(
        "lambda_compose_many",
        None,
        &lambda::rules(),
        "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1))
          (app (app (var compose) (var add1))
               (app (app (var compose) (var add1))
                    (app (app (var compose) (var add1))
                         (app (app (var compose) (var add1))
                              (app (app (var compose) (var add1))
                                   (var add1)))))))))".parse().unwrap(),
        &["(lam ?x (+ (var ?x) 7))".parse().unwrap()],
        None,
        true
    );
}


// #[cfg(not(debug_assertions))]
// #[cfg_attr(feature = "test-explanations", ignore)]
fn lambda_function_repeat() {
    egg::test::test_runner(
        "lambda_function_repeat",
        Some(Runner::default()
            .with_time_limit(std::time::Duration::from_secs(20))
            .with_node_limit(150_000)
            .with_iter_limit(60)),
        &lambda::rules(),
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
          2))))".parse().unwrap(),
        &["(lam ?x (+ (var ?x) 2))".parse().unwrap()],
        None,
        true
    );
}

fn lambda_if() {
    egg::test::test_runner(
        "lambda_if",
        None,
        &lambda::rules(),
        "(let zeroone (lam x
        (if (= (var x) 0)
            0
            1))
        (+ (app (var zeroone) 0)
        (app (var zeroone) 10)))".parse().unwrap(),
        // "(+ (if false 0 1) (if true 0 1))",
        // "(+ 1 0)",
        &["1".parse().unwrap()],
        None,
        true
    );
}

// #[cfg(not(debug_assertions))]
// #[cfg_attr(feature = "test-explanations", ignore)]
fn lambda_fib() {
    egg::test::test_runner(
        "lambda_fib",
        Some(Runner::default()
            .with_iter_limit(60)
            .with_node_limit(500_000)),
        &lambda::rules(),
        "(let fib (fix fib (lam n
        (if (= (var n) 0)
            0
        (if (= (var n) 1)
            1
        (+ (app (var fib)
                (+ (var n) -1))
            (app (var fib)
                (+ (var n) -2)))))))
        (app (var fib) 4))".parse().unwrap(),
        &["3".parse().unwrap()],
        None,
        true
    );
}


pub fn lambda_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambda_tests");
    group.bench_function(
        "lambda_under",
        |b| b.iter(lambda_under)
    );
    group.bench_function(
        "lambda_if_elim",
        |b| b.iter(lambda_if_elim)
    );
    group.bench_function(
        "lambda_let_simple",
        |b| b.iter(lambda_let_simple)
    );
    group.bench_function(
        "lambda_compose",
        |b| b.iter(lambda_compose)
    );
    group.bench_function(
        "lambda_if_simple",
        |b| b.iter(lambda_if_simple)
    );
    group.bench_function(
        "lambda_compose_many",
        |b| b.iter(lambda_compose_many)
    );
    group.bench_function(
        "lambda_function_repeat",
        |b| b.iter(lambda_function_repeat)
    );
    group.bench_function(
        "lambda_if",
        |b| b.iter(lambda_if)
    );
    group.bench_function(
        "lambda_fib",
        |b| b.iter(lambda_fib)
    );
    group.finish();
}



fn lambda_let_simple_iterator() {
    egg::test::test_runner(
        "lambda_let_simple",
        Some(Runner::default()
            .with_scheduler(IteratorScheduler)),
        &lambda::rules(),
        "(let x 0
     (let y 1
     (+ (var x) (var y))))".parse().unwrap(),
        &["1".parse().unwrap()],
        None,
        true
    );
}

fn lambda_let_simple_parallel_iterator() {
    egg::test::test_runner(
        "lambda_let_simple",
        Some(Runner::default()
            .with_scheduler(ParallelIteratorScheduler)),
        &lambda::rules(),
        "(let x 0
     (let y 1
     (+ (var x) (var y))))".parse().unwrap(),
        &["1".parse().unwrap()],
        None,
        true
    );
}

fn lambda_compose_iterator() {
    egg::test::test_runner(
        "lambda_compose",
        Some(Runner::default()
            .with_scheduler(IteratorScheduler)),
        &lambda::rules(),
        "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1)) (var add1))))".parse().unwrap(),
        &[
            "(lam ?x (+ 1
                (app (lam ?y (+ 1 (var ?y)))
                     (var ?x))))".parse().unwrap(),
                "(lam ?x (+ (var ?x) 2))".parse().unwrap()
        ],
        None,
        true
    );
}

fn lambda_compose_parallel_iterator() {
    egg::test::test_runner(
        "lambda_compose",
        Some(Runner::default()
            .with_scheduler(ParallelIteratorScheduler)),
        &lambda::rules(),
        "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1)) (var add1))))".parse().unwrap(),
        &[
            "(lam ?x (+ 1
                (app (lam ?y (+ 1 (var ?y)))
                     (var ?x))))".parse().unwrap(),
                "(lam ?x (+ (var ?x) 2))".parse().unwrap()
        ],
        None,
        true
    );
}

fn lambda_function_repeat_iterator(repeats: i32) {
    egg::test::test_runner(
        "lambda_function_repeat",
        Some(Runner::default()
            .with_scheduler(IteratorScheduler)
            .with_time_limit(std::time::Duration::from_secs(20))
            .with_node_limit(25_000_000)
            .with_iter_limit(60)),
        &lambda::rules(),
        format!("(let compose (lam f (lam g (lam x (app (var f)
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
          {repeats}))))").parse().unwrap(),
        &[format!("(lam ?x (+ (var ?x) {repeats}))").parse().unwrap()],
        None,
        true
    );
}

fn lambda_function_repeat_parallel_iterator(repeats: i32) {
    egg::test::test_runner(
        "lambda_function_repeat",
        Some(Runner::default()
            .with_scheduler(ParallelIteratorScheduler)
            .with_time_limit(std::time::Duration::from_secs(20))
            .with_node_limit(25_000_000)
            .with_iter_limit(60)),
            &lambda::rules(),
            format!("(let compose (lam f (lam g (lam x (app (var f)
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
              {repeats}))))").parse().unwrap(),
            &[format!("(lam ?x (+ (var ?x) {repeats}))").parse().unwrap()],
        None,
        true
    );
}


pub fn lambda_test_serial(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambda_test_serial");
    group.sample_size(10); // Bound the number of samples to avoid overwhelming profiler
    group.bench_function(
        "lambda_function_repeat_iterator",
        |b| b.iter(|| lambda_function_repeat_iterator(3))
    );
    group.finish();
}

pub fn lambda_test_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambda_test_parallel");
    group.sample_size(10); // Bound the number of samples to avoid overwhelming profiler
    group.bench_function(
        "lambda_function_repeat_parallel_iterator",
        |b| b.iter(|| lambda_function_repeat_parallel_iterator(3))
    );
    group.finish();
}

pub fn lambda_test_comparison_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambda_test_comparison_large");
    // group.bench_function(
    //     "lambda_function_repeat_iterator",
    //     |b| b.iter(lambda_function_repeat_iterator(3))
    // );
    // group.bench_function(
    //     "lambda_function_repeat_parallel_iterator",
    //     |b| b.iter(|| lambda_function_repeat_parallel_iterator(3))
    // );
    group.sample_size(10); // Bound the number of samples to avoid overwhelming profiler
    for i in 2..6 {
        group.bench_with_input(BenchmarkId::new("lambda_function_repeat_iterator", i), &i,
        |b, i| b.iter(|| lambda_function_repeat_iterator(*i)));
        group.bench_with_input(BenchmarkId::new("lambda_function_repeat_parallel_iterator", i), &i,
        |b, i| b.iter(|| lambda_function_repeat_parallel_iterator(*i)));
    }
    group.finish();
}

pub fn lambda_test_comparison_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambda_test_comparison_small");
    group.bench_function(
        "lambda_compose_iterator",
        |b| b.iter(lambda_compose_iterator)
    );
    group.bench_function(
        "lambda_compose_parallel_iterator",
        |b| b.iter(lambda_compose_parallel_iterator)
    );
    group.bench_function(
        "lambda_let_simple_iterator",
        |b| b.iter(lambda_let_simple_iterator)
    );
    group.bench_function(
        "lambda_let_simple_parallel_iterator",
        |b| b.iter(lambda_let_simple_parallel_iterator)
    );
    group.finish();

}

criterion_group!(
    benches,
    // lambda_tests,
    // lambda_test_serial,
    // lambda_test_parallel,
    // lambda_test_comparison_small,
    lambda_test_comparison_large
);
criterion_main!(benches);
