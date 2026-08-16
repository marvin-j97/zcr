pub(crate) mod builder;
mod mutator;
pub(crate) mod streaming;

use crate::{
    ValueAccessor,
    list::{BorrowedList, OwnedList},
};
use byteorder::{BE, ReadBytesExt};
use byteview::ByteView;

pub use builder::RecordBuilder;
pub use mutator::Mutator;

const KEY_SLOT_SIZE: usize = std::mem::size_of::<u32>();

/// A zero-copy record borrowing a byte slice
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BorrowedRecord<'a>(pub(crate) &'a [u8]);

impl std::fmt::Debug for BorrowedRecord<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();

        for (key, value) in self.iter() {
            map.entry(&key, &value);
        }

        map.finish()
    }
}

impl<'a> BorrowedRecord<'a> {
    pub fn mutate(&'a self, f: impl FnOnce(&mut Mutator) -> ()) -> OwnedRecord {
        let mut m = Mutator::new(self);
        f(&mut m);
        m.finish()
    }

    pub fn from_slice(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Creates an owned clone of this record.
    pub fn to_owned(&self) -> OwnedRecord {
        OwnedRecord(ByteView::from(self.0))
    }

    fn get_list_end_offset(&self) -> usize {
        let offset =
            // Tag
            std::mem::size_of::<u8>()
            // Keys offset
            + std::mem::size_of::<u32>()
            // Key slots offset
            + std::mem::size_of::<u32>();

        let start = self.0.len() - offset;
        usize::try_from((&self.0[start..]).read_u32::<BE>().unwrap()).unwrap()
    }

    pub fn as_values(&self) -> BorrowedList<'a> {
        let end = self.get_list_end_offset();
        BorrowedList(&self.0[0..end])
    }

    fn search_slot_index(&self, key: &[u8]) -> Option<usize> {
        self.search_by(|h| h.cmp(key)).ok()
    }

    /// Returns the given field, if it exists.
    pub fn get(&self, key: &[u8]) -> Option<ValueAccessor<'a>> {
        self.search_slot_index(key)
            .and_then(|idx| self.as_values().get(idx))
    }

    /// Returns `true` if the field exists.
    pub fn has(&self, key: &[u8]) -> bool {
        self.search_by(|h| h.cmp(key)).is_ok()
    }

    /// Returns the number of fields.
    pub fn len(&self) -> usize {
        self.as_values().len()
    }

    /// Returns an iterator over all field keys.
    pub fn keys(&self) -> impl Iterator<Item = &'_ [u8]> {
        let key_slots_offset = self.get_key_slots_offset();

        (0..self.len()).map(move |idx| self.get_key_at(key_slots_offset, idx))
    }

    /// Returns an iterator over all fields.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], ValueAccessor<'_>)> {
        self.keys().map(|k| {
            let v = self.get(k).unwrap();
            (k, v)
        })
    }

    /// Returns an iterator over all field values.
    pub fn values(&self) -> impl Iterator<Item = ValueAccessor<'_>> {
        self.iter().map(|(_, v)| v)
    }

    fn get_key_slots_offset(&self) -> usize {
        let offset =
            // Tag
            std::mem::size_of::<u8>()
            // Key slots offset
            + std::mem::size_of::<u32>();

        (&self.0[self.0.len() - offset..])
            .read_u32::<BE>()
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn search_by(&self, f: impl Fn(&[u8]) -> std::cmp::Ordering) -> Result<usize, usize> {
        use std::cmp::Ordering::{Equal, Greater, Less};

        let size = self.len();
        if size == 0 {
            return Err(0);
        }

        let key_slots_offset = self.get_key_slots_offset();

        if size <= 5 {
            for idx in 0..size {
                let key = self.get_key_at(key_slots_offset, idx);

                if f(key).is_eq() {
                    return Ok(idx);
                }
            }

            return Err(size);
        }

        let mut left = 0;
        let mut right = size;

        while left < right {
            let mid = left + (right - left) / 2;

            // Reusing the logic to extract the key at 'mid'
            let key = self.get_key_at(key_slots_offset, mid);

            match f(key) {
                Less => left = mid + 1,
                Greater => right = mid,
                Equal => return Ok(mid),
            }
        }

        Err(left)
    }

    fn get_key_indexes(&self, key_slots_offset: usize, idx: usize) -> (usize, usize) {
        let slot_offset = key_slots_offset + idx * KEY_SLOT_SIZE;

        let offset: usize = (&self.0[slot_offset..])
            .read_u32::<BE>()
            .unwrap()
            .try_into()
            .unwrap();

        let data_start = offset;

        let key_len: usize = (&self.0[data_start..])
            .read_u16::<BE>()
            .unwrap()
            .try_into()
            .unwrap();

        let key_offset = data_start + 2;

        (key_offset, key_offset + key_len)
    }

    fn get_key_at(&self, key_slots_offset: usize, idx: usize) -> &[u8] {
        let (lo, hi) = self.get_key_indexes(key_slots_offset, idx);
        &self.0[lo..hi]
    }
}

/// A zero-copy record which owns its byte buffer
// TODO: shared record? -> OwnedRecord would be a Box<[u8]> or Vec<u8>
#[derive(Clone, PartialEq, Eq)]
pub struct OwnedRecord(pub(crate) ByteView);

impl std::fmt::Debug for OwnedRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_borrowed().fmt(f)
    }
}

impl OwnedRecord {
    /// Borrows the inner raw byte slice representing this record.
    pub fn inner(&self) -> &[u8] {
        &self.0
    }

    /// Returns the raw byte slice representing this record.
    pub fn into_inner(self) -> ByteView {
        self.0
    }

    /// Converts the record into a list.
    pub fn into_values(&self) -> OwnedList {
        let end = self.as_borrowed().get_list_end_offset();
        OwnedList(self.0.slice(0..end))
    }

    pub fn as_borrowed(&'_ self) -> BorrowedRecord<'_> {
        BorrowedRecord(&self.0)
    }

    /// Allows accessing the record contents using a List interface.
    pub fn as_values(&self) -> BorrowedList<'_> {
        let end: usize = self.as_borrowed().get_list_end_offset();
        BorrowedList(&self.0[0..end])
    }

    /// Returns the given field, if it exists.
    pub fn get(&self, key: &[u8]) -> Option<ValueAccessor<'_>> {
        self.as_borrowed().get(key)
    }

    /// Returns `true` if the field exists.
    pub fn has(&self, key: &[u8]) -> bool {
        self.as_borrowed().has(key)
    }

    /// Returns the number of fields.
    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    /// Returns an iterator over all field keys.
    pub fn keys(&self) -> impl Iterator<Item = &'_ [u8]> {
        let b = self.as_borrowed();

        let key_slots_offset = b.get_key_slots_offset();

        (0..self.len()).map(move |idx| {
            let (lo, hi) = b.get_key_indexes(key_slots_offset, idx);
            &self.0[lo..hi]
        })
    }

    /// Returns an iterator over all fields.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], ValueAccessor<'_>)> {
        self.keys().map(|key| {
            let v = self.get(key).unwrap();
            (key, v)
        })
    }

    /// Returns an iterator over all field values.
    pub fn values(&self) -> impl Iterator<Item = ValueAccessor<'_>> {
        self.iter().map(|(_, v)| v)
    }
}
