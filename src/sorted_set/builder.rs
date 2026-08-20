use crate::{OwnedList, OwnedSortedSet, borrowed_value::BorrowedValue};
use std::collections::BTreeSet;

/// Sorted set builder
pub struct SortedSetBuilder {
    inner: BTreeSet<Vec<u8>>,
}

impl Default for SortedSetBuilder {
    fn default() -> Self {
        Self {
            inner: BTreeSet::default(),
        }
    }
}

impl SortedSetBuilder {
    pub fn insert(&mut self, value: impl Into<Vec<u8>>) {
        self.inner.insert(value.into());
    }

    pub fn finish(self) -> OwnedSortedSet {
        let mut v = vec![];
        {
            let mut list_builder = crate::ListBuilder::new(&mut v);

            for item in self.inner {
                list_builder.push(BorrowedValue::Bytes(&*item));
            }

            list_builder.finish();
        }

        OwnedSortedSet::new(OwnedList(v.into()))
    }
}
