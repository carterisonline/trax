use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SRC: &str = include_str!("../todo.trax");

fn parse(text: &str) -> Result<(), trax_parser::Error> {
    for token in trax_parser::Tokenizer::from(text) {
        black_box(token?);
    }

    Ok(())
}

fn bench(c: &mut Criterion) {
    let mut parse_group = c.benchmark_group("parse");

    parse_group.bench_function("todo", |b| b.iter(|| parse(black_box(SRC))));
    parse_group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench
);

criterion_main!(benches);
