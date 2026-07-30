use criterion::{Criterion, criterion_group, criterion_main};
use zcr::{RecordBuilder, WrappedValue};

fn record_mutate_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("Record mutate");

    for item_count in [1u64, 5, 10, 100, 1_000] {
        let record = {
            let mut builder = RecordBuilder::default();

            for item in 0..item_count {
                builder = builder.prop(
                    item.to_be_bytes(),
                    WrappedValue::String(format!("hello world {item}")),
                );
            }

            builder.finish()
        };

        group.bench_function(format!("{item_count} items"), |b| {
            b.iter(|| {
                record.as_borrowed().mutate(|m| {
                    m.insert("abcdef", "ghijk");
                })
            })
        });
    }
}

criterion_group!(benches, record_mutate_simple);
criterion_main!(benches);
