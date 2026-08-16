use crate::{
    BorrowedRecord, OwnedRecord, WrappedValue,
    borrowed_value::BorrowedValue,
    record::{builder::RecordKey, streaming::StreamingRecordBuilder},
};
use byteview::ByteView;
use std::collections::BTreeMap;

enum Node<'a> {
    Ref(&'a [u8]),
    Value(WrappedValue),
}

enum Key<'a> {
    Ref(&'a [u8]),
    Owned(ByteView),
}

impl Key<'_> {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Key::Ref(bytes) => bytes,
            Key::Owned(bytes) => bytes.as_ref(),
        }
    }
}

impl PartialEq for Key<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Key<'_> {}

impl Ord for Key<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Key<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (Key::Ref(a), Key::Ref(b)) => a.cmp(b),
            (Key::Ref(a), Key::Owned(b)) => a.cmp(&b.as_ref()),
            (Key::Owned(a), Key::Ref(b)) => a.as_ref().cmp(b),
            (Key::Owned(a), Key::Owned(b)) => a.as_ref().cmp(b.as_ref()),
        })
    }
}

impl std::borrow::Borrow<[u8]> for Key<'_> {
    fn borrow(&self) -> &[u8] {
        match self {
            Key::Ref(bytes) => bytes,
            Key::Owned(bytes) => bytes.as_ref(),
        }
    }
}

pub struct Mutator<'a> {
    state: BTreeMap<Key<'a>, Node<'a>>,
}

impl<'a> Mutator<'a> {
    pub(crate) fn new(record: &'a BorrowedRecord<'a>) -> Self {
        let mut state = BTreeMap::default();

        for (k, v) in record.iter() {
            state.insert(Key::Ref(k), Node::Ref(v.inner()));
        }

        Self { state }
    }

    pub fn insert(&mut self, key: impl Into<RecordKey>, value: impl Into<WrappedValue>) {
        self.state
            .insert(Key::Owned(key.into()), Node::Value(value.into()));
    }

    pub fn remove(&mut self, key: &[u8]) {
        self.state.remove(key);
    }

    pub fn clear(&mut self) {
        self.state.clear();
    }

    pub fn finish(self) -> OwnedRecord {
        let mut v = vec![];
        let mut builder = StreamingRecordBuilder::new(&mut v);

        for (k, v) in self.state {
            match v {
                Node::Ref(r) => {
                    builder.push(
                        match k {
                            Key::Ref(k) => k.into(),
                            Key::Owned(k) => k,
                        },
                        BorrowedValue::Raw(&*r),
                    );
                }
                Node::Value(v) => {
                    builder.push(
                        match k {
                            Key::Ref(k) => k.into(),
                            Key::Owned(k) => k,
                        },
                        &v,
                    );
                }
            }
        }

        builder.finish();

        OwnedRecord(v.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::RecordBuilder;
    use test_log::test;

    #[test]
    fn record_mutate_identity() {
        let rec = RecordBuilder::default()
            .prop("a", 1u8)
            .prop("b", "hello")
            .prop("is_true", true)
            .prop("numbers", [1, 2, 3, 4, 5])
            .finish();

        let new_rec = rec.as_borrowed().mutate(|_| {});
        assert_eq!(rec, new_rec)
    }

    #[test]
    fn record_mutate_remove() {
        let rec = RecordBuilder::default()
            .prop("a", 1u8)
            .prop("b", "hello")
            .prop("is_true", true)
            .prop("numbers", [1, 2, 3, 4, 5])
            .finish();

        let new_rec = rec.as_borrowed().mutate(|m| {
            m.remove(b"b");
            m.remove(b"is_true");
        });

        assert!(new_rec.has(b"a"));
        assert!(!new_rec.has(b"b"));
        assert!(!new_rec.has(b"is_true"));
        assert!(new_rec.has(b"numbers"));
    }

    #[test]
    fn record_mutate_insert() {
        let rec = RecordBuilder::default()
            .prop("a", 1u8)
            .prop("b", "hello")
            .prop("is_true", true)
            .prop("numbers", [1, 2, 3, 4, 5])
            .finish();

        let new_rec = rec.as_borrowed().mutate(|m| {
            m.insert("c", *b"test");
        });

        assert!(new_rec.has(b"a"));
        assert!(new_rec.has(b"b"));
        assert!(new_rec.has(b"c"));
        assert!(new_rec.has(b"is_true"));
        assert!(new_rec.has(b"numbers"));
    }
}
