use criterion::{Criterion, criterion_group, criterion_main};
use rand::RngExt;
use zcr::{ListBuilder, RecordBuilder, WrappedValue};

fn record_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("Record get item");

    for item_count in [1u64, 5, 10, 100, 1_000, 10_000] {
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

        let mut rng = rand::rng();

        group.bench_function(format!("{item_count} items"), |b| {
            b.iter(|| {
                let key = rng.random_range(0..item_count).to_be_bytes();
                record.get(&key)
            })
        });
    }
}

fn list_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("List get item");

    for item_count in [10usize, 100, 1_000, 10_000] {
        let record = {
            let mut v = vec![];
            let mut builder = ListBuilder::new(&mut v);

            for item in 0..item_count {
                builder.push(&*format!("hello world {item}"));
            }

            builder.finish()
        };

        let mut rng = rand::rng();

        group.bench_function(format!("{item_count} items"), |b| {
            b.iter(|| {
                let idx = rng.random_range(0..item_count);
                record.get(idx)
            })
        });
    }
}

criterion_group!(benches, record_get, list_get);
criterion_main!(benches);
