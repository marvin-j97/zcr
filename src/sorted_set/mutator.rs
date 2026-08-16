use crate::{
    BorrowedSortedSet, ListBuilder, OwnedList, OwnedSortedSet, borrowed_value::BorrowedValue,
};
use std::collections::BTreeSet;

enum Key<'a> {
    Ref(&'a [u8]),
    Owned(Vec<u8>),
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
        Some(self.as_bytes().cmp(other.as_bytes()))
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
    state: BTreeSet<Key<'a>>,
}

impl<'a> Mutator<'a> {
    pub(crate) fn new(record: &'a BorrowedSortedSet<'a>) -> Self {
        let mut state = BTreeSet::default();

        for k in record.iter() {
            state.insert(Key::Ref(k.as_raw_bytes()));
        }

        Self { state }
    }

    pub fn insert(&mut self, key: impl Into<Vec<u8>>) {
        self.state.insert(Key::Owned(key.into()));
    }

    pub fn remove(&mut self, key: &[u8]) {
        self.state.remove(key);
    }

    pub fn clear(&mut self) {
        self.state.clear();
    }

    pub fn finish(self) -> OwnedSortedSet {
        let mut v = vec![];
        let mut builder = ListBuilder::new(&mut v);

        for k in self.state {
            builder.push(BorrowedValue::Bytes(k.as_bytes()));
        }

        builder.finish();

        OwnedSortedSet::new(OwnedList(v.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::SortedSetBuilder;
    use test_log::test;

    #[test]
    fn sset_mutate_identity() {
        let mut rec = SortedSetBuilder::default();
        rec.insert(b"abc");
        rec.insert(b"hello");
        let rec = rec.finish();

        let new_rec = rec.as_borrowed().mutate(|_| {});
        assert_eq!(rec, new_rec)
    }

    #[test]
    fn sset_mutate_remove() {
        let mut rec = SortedSetBuilder::default();
        rec.insert(b"abc");
        rec.insert(b"hello");
        let rec = rec.finish();

        let new_rec = rec.as_borrowed().mutate(|m| {
            m.remove(b"abc");
        });

        assert!(!new_rec.has(b"abc"));
        assert!(new_rec.has(b"hello"));
    }

    #[test]
    fn sset_mutate_insert() {
        let mut rec = SortedSetBuilder::default();
        rec.insert(b"abc");
        rec.insert(b"hello");
        let rec = rec.finish();

        let new_rec = rec.as_borrowed().mutate(|m| {
            m.insert("c");
        });

        assert!(new_rec.has(b"abc"));
        assert!(new_rec.has(b"hello"));
        assert!(new_rec.has(b"c"));
    }
}
