mod builder;
mod mutator;

use crate::{BorrowedList, OwnedList, ValueAccessor};

pub use builder::SortedSetBuilder;
pub use mutator::Mutator;

/// A zero-copy, sorted byte set borrowing a byte slice
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BorrowedSortedSet<'a> {
    inner: BorrowedList<'a>,
}

impl<'a> BorrowedSortedSet<'a> {
    pub fn from_slice(slice: &'a [u8]) -> Self {
        Self {
            inner: BorrowedList::from_slice(slice),
        }
    }

    /// Returns the raw serialized representation.
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    pub fn mutate(&'a self, f: impl FnOnce(&mut Mutator<'a>) -> ()) -> OwnedSortedSet {
        let mut m = Mutator::new(self);
        f(&mut m);
        m.finish()
    }

    pub fn as_list(&self) -> BorrowedList<'a> {
        self.inner.clone()
    }

    pub fn iter(&self) -> impl Iterator<Item = ValueAccessor<'_>> {
        self.inner.iter()
    }

    // TODO: rename contains()
    pub fn has<'b>(&self, k: impl AsRef<[u8]>) -> bool {
        let k = k.as_ref();

        self.inner
            .binary_search_by(|v| v.as_bytes().expect("should be bytes").cmp(k))
            .is_ok()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// A zero-copy, sorted byte set that owns its byte buffer
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedSortedSet {
    inner: OwnedList,
}

impl OwnedSortedSet {
    pub(crate) fn new(list: OwnedList) -> Self {
        Self { inner: list }
    }

    pub fn as_borrowed(&self) -> BorrowedSortedSet<'_> {
        BorrowedSortedSet {
            inner: self.inner.as_borrowed(),
        }
    }

    pub fn as_list(&self) -> BorrowedList<'_> {
        self.inner.as_borrowed()
    }

    pub fn into_list(self) -> OwnedList {
        self.inner
    }

    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    pub fn iter(&self) -> impl Iterator<Item = ValueAccessor<'_>> {
        self.inner.iter()
    }

    pub fn has<'b>(&self, key: &[u8]) -> bool {
        self.as_borrowed().has(key)
    }
}

#[cfg(test)]
mod tests {
    use crate::SortedSetBuilder;
    use test_log::test;

    #[test]
    fn sorted_set_smoke_test() {
        let mut set = SortedSetBuilder::default();
        set.insert(b"hello");
        set.insert(b"bye");
        set.insert(b"whatever");
        let set = set.finish();

        assert_eq!(3, set.len(), "should have 3 items");

        assert!(set.has(b"hello"), "should have item");
        assert!(set.has(b"bye"), "should have item");
        assert!(set.has(b"whatever"), "should have item");

        assert!(!set.has(b"nonexistent"), "should not have item");

        {
            let it = set.iter().map(|v| v.as_bytes().unwrap());
            let items = it.collect::<Vec<_>>();
            assert_eq!(vec![b"bye" as &[u8], b"hello", b"whatever"], items);
        }
    }
}
