use crate::{BorrowedRecord, list::BorrowedList, value_tag::ValueTag};
use byteorder::{BE, ReadBytesExt};

/// Allows reading a value from a record or list
///
/// Each value in a record/list is tagged by its type and needs to be cast to the correct type.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ValueAccessor<'a>(&'a [u8]);

impl<'a> ValueAccessor<'a> {
    pub(crate) fn new(slice: &'a [u8]) -> Self {
        Self(slice)
    }
}

impl std::fmt::Debug for ValueAccessor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value_type() {
            ValueTag::Null => f.write_str("NULL"),
            ValueTag::U8 => self.as_u8().unwrap().fmt(f),
            ValueTag::U16 => self.as_u16().unwrap().fmt(f),
            ValueTag::U32 => self.as_u32().unwrap().fmt(f),
            ValueTag::U64 => self.as_u64().unwrap().fmt(f),
            ValueTag::U128 => self.as_u128().unwrap().fmt(f),
            ValueTag::I8 => self.as_i8().unwrap().fmt(f),
            ValueTag::I16 => self.as_i16().unwrap().fmt(f),
            ValueTag::I32 => self.as_i32().unwrap().fmt(f),
            ValueTag::I64 => self.as_i64().unwrap().fmt(f),
            ValueTag::I128 => self.as_i128().unwrap().fmt(f),
            ValueTag::F32 => self.as_f32().unwrap().fmt(f),
            ValueTag::F64 => self.as_f64().unwrap().fmt(f),
            ValueTag::Boolean => self.as_bool().unwrap().fmt(f),
            ValueTag::Bytes => self.as_bytes().unwrap().fmt(f),
            ValueTag::String => self.as_str().unwrap().fmt(f),
            ValueTag::List => self.as_list().unwrap().fmt(f),
            ValueTag::Record => self.as_record().unwrap().fmt(f),
        }
    }
}

impl<'a> ValueAccessor<'a> {
    pub fn value_type(&self) -> ValueTag {
        match self.0.last().unwrap() {
            0 => ValueTag::Null,
            1 => ValueTag::U8,
            2 => ValueTag::U16,
            3 => ValueTag::U32,
            4 => ValueTag::U64,
            5 => ValueTag::U128,
            6 => ValueTag::I8,
            7 => ValueTag::I16,
            8 => ValueTag::I32,
            9 => ValueTag::I64,
            10 => ValueTag::I128,
            11 => ValueTag::F32,
            12 => ValueTag::F64,
            13 => ValueTag::Boolean,
            14 => ValueTag::Bytes,
            15 => ValueTag::String,
            16 => ValueTag::List,
            17 => ValueTag::Record,
            _ => panic!("invalid value tag"),
        }
    }

    pub fn is_null(&self) -> bool {
        self.value_type() == ValueTag::Null
    }

    pub(crate) fn inner(&self) -> &'a [u8] {
        &self.0
    }

    pub fn as_raw_bytes(&self) -> &'a [u8] {
        &self.0[0..self.0.len() - 1]
    }

    pub fn as_list(&'_ self) -> Option<BorrowedList<'a>> {
        if self.value_type() != ValueTag::List {
            None
        } else {
            Some(BorrowedList(&self.0))
        }
    }

    pub fn as_record(&'_ self) -> Option<BorrowedRecord<'a>> {
        if self.value_type() != ValueTag::Record {
            None
        } else {
            Some(BorrowedRecord(&self.0))
        }
    }

    pub fn as_bytes(&self) -> Option<&'a [u8]> {
        if self.value_type() != ValueTag::Bytes {
            None
        } else {
            let hi = self.0.len() - 1;
            Some(&self.0[0..hi])
        }
    }

    pub fn as_str(&self) -> Option<&'a str> {
        if self.value_type() != ValueTag::String {
            None
        } else {
            let hi = self.0.len() - 1;
            Some(std::str::from_utf8(&self.0[0..hi]).unwrap())
        }
    }

    pub unsafe fn as_str_unchecked(&self) -> Option<&'a str> {
        if self.value_type() != ValueTag::String {
            None
        } else {
            let hi = self.0.len() - 1;
            Some(unsafe { std::str::from_utf8_unchecked(&self.0[0..hi]) })
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if self.value_type() != ValueTag::Boolean {
            None
        } else {
            Some(self.0.first().copied().unwrap() > 0)
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        if self.value_type() != ValueTag::F32 {
            None
        } else {
            Some((&self.0[..]).read_f32::<BE>().unwrap())
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if self.value_type() != ValueTag::F64 {
            None
        } else {
            Some((&self.0[..]).read_f64::<BE>().unwrap())
        }
    }

    pub fn as_u8(&self) -> Option<u8> {
        if self.value_type() != ValueTag::U8 {
            None
        } else {
            Some(self.0.first().copied().unwrap())
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        if self.value_type() != ValueTag::U16 {
            None
        } else {
            Some((&self.0[..]).read_u16::<BE>().unwrap())
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        if self.value_type() != ValueTag::U32 {
            None
        } else {
            Some((&self.0[..]).read_u32::<BE>().unwrap())
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if self.value_type() != ValueTag::U64 {
            None
        } else {
            Some((&self.0[..]).read_u64::<BE>().unwrap())
        }
    }

    pub fn as_u128(&self) -> Option<u128> {
        if self.value_type() != ValueTag::U128 {
            None
        } else {
            Some((&self.0[..]).read_u128::<BE>().unwrap())
        }
    }

    pub fn as_i8(&self) -> Option<i8> {
        if self.value_type() != ValueTag::I8 {
            None
        } else {
            Some(self.0.first().copied().unwrap() as i8)
        }
    }

    pub fn as_i16(&self) -> Option<i16> {
        if self.value_type() != ValueTag::I16 {
            None
        } else {
            Some((&self.0[..]).read_i16::<BE>().unwrap())
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        if self.value_type() != ValueTag::I32 {
            None
        } else {
            Some((&self.0[..]).read_i32::<BE>().unwrap())
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if self.value_type() != ValueTag::I64 {
            None
        } else {
            Some((&self.0[..]).read_i64::<BE>().unwrap())
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        if self.value_type() != ValueTag::I128 {
            None
        } else {
            Some((&self.0[..]).read_i128::<BE>().unwrap())
        }
    }
}
