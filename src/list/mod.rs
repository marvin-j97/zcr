mod builder;
mod mutator;

use crate::{ValueAccessor, list::builder::ValueOffset};
use byteorder::{BE, ReadBytesExt};
use byteview::ByteView;

pub use builder::ListBuilder;
pub use mutator::Mutator;

/// A zero-copy array which owns its byte buffer
#[derive(Clone, PartialEq, Eq)]
pub struct OwnedList(pub(crate) ByteView);

impl std::fmt::Debug for OwnedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        BorrowedList(&self.0).fmt(f)
    }
}

impl OwnedList {
    /// Returns the raw byte slice representing this list.
    pub fn into_inner(self) -> ByteView {
        self.0
    }

    pub fn as_borrowed(&self) -> BorrowedList<'_> {
        BorrowedList(&self.0)
    }

    /// Returns the number of list items.
    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    /// Returns the first field, if it exists.
    pub fn first(&self) -> Option<ValueAccessor<'_>> {
        self.as_borrowed().first()
    }
    /// Returns the first field, if it exists.
    pub fn last(&self) -> Option<ValueAccessor<'_>> {
        self.as_borrowed().last()
    }
    /// Returns the n-th field, if it exists.
    pub fn get(&self, idx: usize) -> Option<ValueAccessor<'_>> {
        self.as_borrowed().get(idx)
    }

    #[doc(hidden)]
    pub fn binary_search_by<F: Fn(&ValueAccessor<'_>) -> std::cmp::Ordering>(
        &self,
        f: F,
    ) -> Result<usize, usize> {
        self.as_borrowed().binary_search_by(f)
    }

    pub fn iter(&self) -> impl Iterator<Item = ValueAccessor<'_>> {
        (0..self.len()).map(|idx| self.get(idx)).flatten()
    }

    pub fn slice<R: std::ops::RangeBounds<usize>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = ValueAccessor<'_>> {
        use std::ops::Bound::{Excluded, Included, Unbounded};

        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n + 1,
            Unbounded => 0,
        };

        let end = match range.end_bound() {
            Included(&n) => n + 1,
            Excluded(&n) => n,
            Unbounded => self.len(),
        };

        let safe_start = start.min(self.len());
        let safe_end = end.min(self.len()).max(safe_start);

        (safe_start..safe_end).map(|idx| self.get(idx)).flatten()
    }
}

/// A zero-copy array borrowing a byte slice
#[derive(Clone, PartialEq, Eq)]
pub struct BorrowedList<'a>(pub(crate) &'a [u8]);

impl std::fmt::Debug for BorrowedList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();

        for v in self.iter() {
            list.entry(&v);
        }

        list.finish()
    }
}

impl<'a> BorrowedList<'a> {
    pub fn mutate(&'a self, f: impl FnOnce(&mut Mutator) -> ()) -> OwnedList {
        let mut m = Mutator::new(self);
        f(&mut m);
        m.finish()
    }

    pub fn from_slice(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Creates an owned clone of this record.
    pub fn to_owned(&self) -> OwnedList {
        OwnedList(ByteView::from(self.0))
    }

    /// Returns the raw serialized representation.
    pub fn as_bytes(&self) -> &[u8] {
        self.0
    }

    /// Returns the number of list items.
    pub fn len(&self) -> usize {
        const OFFSET: usize =
            // Tag
            std::mem::size_of::<u8>()
            // Len field
            + std::mem::size_of::<u32>();

        let start = self.0.len() - OFFSET;

        (&self.0[start..])
            .read_u32::<BE>()
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn slot_array_offset(&self) -> usize {
        const OFFSET: usize =
            // Tag
            std::mem::size_of::<u8>()
            // Len field
            + std::mem::size_of::<u32>()
            // Offset field
            + std::mem::size_of::<ValueOffset>();

        let start = self.0.len() - OFFSET;

        usize::try_from((&self.0[start..]).read_u32::<BE>().unwrap()).unwrap()
    }

    fn extract_field_slice_indexes(&self, idx: usize) -> Option<(usize, usize)> {
        const SLOT_SIZE: usize = std::mem::size_of::<u32>() + std::mem::size_of::<ValueOffset>();

        let len = self.len();
        if len == 0 || idx >= len {
            return None;
        }

        let offset = self.slot_array_offset() + SLOT_SIZE * idx;

        let reader = &mut &self.0[offset..];

        let field_offset: usize = reader.read_u32::<BE>().unwrap().try_into().unwrap();
        let field_len: usize = reader.read_u32::<BE>().unwrap().try_into().unwrap();

        Some((field_offset, field_offset + field_len))
    }

    fn extract_field_slice(&self, idx: usize) -> Option<&'a [u8]> {
        let (lo, hi) = self.extract_field_slice_indexes(idx)?;
        Some(&self.0[lo..hi])
    }

    /// Returns the first field, if it exists.
    pub fn first(&self) -> Option<ValueAccessor<'a>> {
        self.get(0)
    }

    /// Returns the first field, if it exists.
    pub fn last(&self) -> Option<ValueAccessor<'a>> {
        let idx = self.len().checked_sub(1)?;
        self.get(idx)
    }

    /// Returns the n-th field, if it exists.
    pub fn get(&self, idx: usize) -> Option<ValueAccessor<'a>> {
        self.extract_field_slice(idx).map(ValueAccessor::new)
    }

    #[doc(hidden)]
    pub fn binary_search_by<F: Fn(&ValueAccessor<'a>) -> std::cmp::Ordering>(
        &self,
        f: F,
    ) -> Result<usize, usize> {
        if self.len() == 0 {
            return Err(0);
        }

        let left = 0;
        let right = self.len() - 1;

        // Binary search
        let mut left = left;
        let mut right = right;

        while left <= right {
            let mid = left + (right - left) / 2;
            let mid_value = self.get(mid).unwrap();

            match f(&mid_value) {
                std::cmp::Ordering::Less => {
                    left = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    if mid == 0 {
                        break;
                    }
                    right = mid - 1;
                }
                std::cmp::Ordering::Equal => {
                    return Ok(mid);
                }
            }
        }

        Err(left)
    }

    pub fn iter(&self) -> impl Iterator<Item = ValueAccessor<'a>> {
        (0..self.len()).map(|idx| self.get(idx)).flatten()
    }

    pub fn slice<R: std::ops::RangeBounds<usize>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = ValueAccessor<'a>> {
        use std::ops::Bound::{Excluded, Included, Unbounded};

        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n + 1,
            Unbounded => 0,
        };

        let end = match range.end_bound() {
            Included(&n) => n + 1,
            Excluded(&n) => n,
            Unbounded => self.len(),
        };

        let safe_start = start.min(self.len());
        let safe_end = end.min(self.len()).max(safe_start);

        (safe_start..safe_end).flat_map(|idx| self.get(idx))
    }
}

#[cfg(test)]
mod tests {
    use crate::{ListBuilder, list::BorrowedList};
    use test_log::test;

    #[test]
    fn list_binary_search_100() {
        let item_count = 100;

        let mut keys = (0..item_count)
            .map(|x| format!("hello world {x}"))
            .collect::<Vec<_>>();

        // Make sure to be lexicographically sorted
        keys.sort();

        let mut buf = vec![];
        {
            let mut builder = ListBuilder::new(&mut buf);

            for k in keys {
                builder.push(&*k);
            }

            builder.finish()
        };
        let list = BorrowedList::from_slice(&buf);

        for x in 0..item_count {
            let key = format!("hello world {x}");

            assert!(
                list.get(
                    list.binary_search_by(|v| {
                        let v = unsafe { v.as_str_unchecked().unwrap() };
                        v.cmp(&key)
                    })
                    .unwrap(),
                )
                .is_some()
            );
        }
    }

    #[test]
    fn list_binary_search() {
        let mut buf = vec![];

        let mut builder = ListBuilder::new(&mut buf);
        builder.push(1u64);
        builder.push(2u64);
        builder.push(3u64);
        builder.push(4u64);
        builder.push(5u64);
        builder.finish();

        let list = BorrowedList(&buf);
        assert_eq!(5, list.len());

        assert_eq!(Some(1), list.first().and_then(|x| x.as_u64()));
        assert_eq!(Some(5), list.last().and_then(|x| x.as_u64()));

        assert_eq!(
            Ok(0),
            list.binary_search_by(|x| x.as_u64().unwrap().cmp(&1)),
        );
        assert_eq!(
            Ok(1),
            list.binary_search_by(|x| x.as_u64().unwrap().cmp(&2)),
        );
        assert_eq!(
            Ok(2),
            list.binary_search_by(|x| x.as_u64().unwrap().cmp(&3)),
        );
        assert_eq!(
            Ok(3),
            list.binary_search_by(|x| x.as_u64().unwrap().cmp(&4)),
        );
        assert_eq!(
            Ok(4),
            list.binary_search_by(|x| x.as_u64().unwrap().cmp(&5)),
        );

        assert!(matches!(
            list.binary_search_by(|x| x.as_u64().unwrap().cmp(&9)),
            Err(_),
        ));
    }

    #[test]
    fn list_simple() {
        let mut buf = vec![];

        let mut builder = ListBuilder::new(&mut buf);
        builder.push("Alice");
        builder.push("Bob");
        builder.push("Charlie");
        builder.finish();

        let list = BorrowedList(&buf);
        assert_eq!(3, list.len());

        assert_eq!(Some("Alice"), list.first().and_then(|x| x.as_str()));
        assert_eq!(Some("Charlie"), list.last().and_then(|x| x.as_str()));

        let middle = list.get(1);
        assert_eq!(Some("Bob"), middle.and_then(|x| x.as_str()));

        {
            let count = list.iter().count();
            assert_eq!(3, count);

            let items = list.iter().collect::<Vec<_>>();
            assert_eq!(Some("Alice"), items.get(0).and_then(|x| x.as_str()));
            assert_eq!(Some("Bob"), items.get(1).and_then(|x| x.as_str()));
            assert_eq!(Some("Charlie"), items.get(2).and_then(|x| x.as_str()));
        }

        {
            let first_two = list.slice(0..2).collect::<Vec<_>>();
            assert_eq!(2, first_two.len());
            assert_eq!(Some("Alice"), first_two.get(0).and_then(|x| x.as_str()));
            assert_eq!(Some("Bob"), first_two.get(1).and_then(|x| x.as_str()));
        }
    }
}
