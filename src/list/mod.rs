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

    /// Returns the number of list items.
    pub fn len(&self) -> usize {
        let offset =
            // Tag
            std::mem::size_of::<u8>()
            // Len field
            + std::mem::size_of::<u32>();

        let start = self.0.len() - offset;

        (&self.0[start..])
            .read_u32::<BE>()
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn slot_array_offset(&self) -> usize {
        let offset =
            // Tag
            std::mem::size_of::<u8>()
            // Len field
            + std::mem::size_of::<u32>()
            // Offset field
            + std::mem::size_of::<ValueOffset>();

        let start = self.0.len() - offset;

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
