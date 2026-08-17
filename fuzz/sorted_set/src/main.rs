use afl::fuzz;
use arbitrary::Arbitrary;
use std::collections::BTreeSet;
use zcr::SortedSetBuilder;

type Key = Vec<u8>;

#[derive(Debug, Arbitrary)]
enum Mutation {
    Insert(String),
    Remove(String),
    Clear,
}

#[derive(Debug, Arbitrary)]
struct Input {
    entries: Vec<Key>,
    mutations: Vec<Mutation>,
}

fn assert_sset_eq(expected: &BTreeSet<Key>, actual: zcr::BorrowedSortedSet<'_>) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "set length mismatch: expected={}, actual={}",
        expected.len(),
        actual.len(),
    );

    for (expected, actual) in expected
        .iter()
        .zip(actual.iter().map(|a| a.as_bytes().unwrap()))
    {
        assert_eq!(expected, actual);
    }
}

fn main() {
    fuzz!(|input: Input| {
        let mut expected = BTreeSet::<Key>::new();
        let mut builder = SortedSetBuilder::default();

        for k in &input.entries {
            expected.insert(k.clone());
            builder.insert(k.clone());
        }

        let mut sset = builder.finish();

        //eprintln!("{expected:#?}");
        //eprintln!("{sset:#?}");

        assert_sset_eq(&expected, sset.as_borrowed());

        for mutation in input.mutations.iter() {
            match mutation {
                Mutation::Insert(k) => {
                    expected.insert(k.as_bytes().to_vec());

                    sset = sset.as_borrowed().mutate(|m| {
                        m.insert(k.clone());
                    });
                }

                Mutation::Remove(k) => {
                    expected.remove(k.as_bytes());

                    sset = sset.as_borrowed().mutate(|m| {
                        m.remove(k.as_bytes());
                    });
                }

                Mutation::Clear => {
                    expected.clear();

                    sset = sset.as_borrowed().mutate(|m| {
                        m.clear();
                    });
                }
            }

            assert_sset_eq(&expected, sset.as_borrowed());
        }
    });
}
