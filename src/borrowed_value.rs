use crate::{ValueTag, WrappedValue, list::ListBuilder, record::streaming::StreamingRecordBuilder};
use std::collections::BTreeMap;

#[derive(Clone)]
pub enum BorrowedValue<'a> {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    F32(f32),
    F64(f64),

    Boolean(bool),

    Bytes(&'a [u8]),
    String(&'a str),

    List(Vec<BorrowedValue<'a>>),
    Record(BTreeMap<&'a [u8], BorrowedValue<'a>>),

    Raw(&'a [u8]),
}

impl<'a> From<BorrowedValue<'a>> for WrappedValue {
    fn from(value: BorrowedValue<'a>) -> Self {
        match value {
            BorrowedValue::U8(x) => Self::U8(x),
            BorrowedValue::U16(x) => Self::U16(x),
            BorrowedValue::U32(x) => Self::U32(x),
            BorrowedValue::U64(x) => Self::U64(x),
            BorrowedValue::U128(x) => Self::U128(x),
            BorrowedValue::I8(x) => Self::I8(x),
            BorrowedValue::I16(x) => Self::I16(x),
            BorrowedValue::I32(x) => Self::I32(x),
            BorrowedValue::I64(x) => Self::I64(x),
            BorrowedValue::I128(x) => Self::I128(x),

            BorrowedValue::F32(x) => Self::F32(x),
            BorrowedValue::F64(x) => Self::F64(x),

            BorrowedValue::Boolean(x) => Self::Boolean(x),

            BorrowedValue::String(x) => Self::String(x.into()),

            BorrowedValue::Bytes(x) => Self::Bytes(x.into()),

            BorrowedValue::List(x) => Self::List(x.into_iter().map(Into::into).collect()),

            BorrowedValue::Record(x) => {
                Self::Map(x.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
            }

            BorrowedValue::Raw(_v) => unreachable!(),
        }
    }
}

impl<'a> From<&'a WrappedValue> for BorrowedValue<'a> {
    fn from(value: &'a WrappedValue) -> Self {
        match value {
            WrappedValue::U8(x) => Self::U8(*x),
            WrappedValue::U16(x) => Self::U16(*x),
            WrappedValue::U32(x) => Self::U32(*x),
            WrappedValue::U64(x) => Self::U64(*x),
            WrappedValue::U128(x) => Self::U128(*x),
            WrappedValue::I8(x) => Self::I8(*x),
            WrappedValue::I16(x) => Self::I16(*x),
            WrappedValue::I32(x) => Self::I32(*x),
            WrappedValue::I64(x) => Self::I64(*x),
            WrappedValue::I128(x) => Self::I128(*x),

            WrappedValue::F32(x) => Self::F32(*x),
            WrappedValue::F64(x) => Self::F64(*x),

            WrappedValue::Boolean(x) => Self::Boolean(*x),

            WrappedValue::String(x) => Self::String(&x),

            WrappedValue::Bytes(x) => Self::Bytes(x),

            WrappedValue::List(x) => Self::List(x.iter().map(Into::into).collect()),

            WrappedValue::Map(x) => Self::Record(
                x.iter()
                    .map(|(k, v)| (&**k, v.into()))
                    .collect::<BTreeMap<_, _>>(),
            ),
        }
    }
}

impl BorrowedValue<'_> {
    pub fn write_into(&self, buf: &mut Vec<u8>) {
        match self {
            Self::U8(v) => {
                buf.push(*v);
                buf.push(ValueTag::U8 as u8);
            }
            Self::U16(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U16 as u8);
            }
            Self::U32(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U32 as u8);
            }
            Self::U64(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U64 as u8);
            }
            Self::U128(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U128 as u8);
            }
            //
            Self::I8(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I8 as u8);
            }
            Self::I16(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I16 as u8);
            }
            Self::I32(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I32 as u8);
            }
            Self::I64(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I64 as u8);
            }
            Self::I128(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I128 as u8);
            }
            //
            Self::F32(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::F32 as u8);
            }
            Self::F64(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::F64 as u8);
            }
            //
            Self::Boolean(v) => {
                buf.push(if *v { 1 } else { 0 });
                buf.push(ValueTag::Boolean as u8);
            }
            //
            Self::Bytes(v) => {
                buf.extend(*v);
                buf.push(ValueTag::Bytes as u8);
            }
            Self::String(s) => {
                buf.extend(s.as_bytes());
                buf.push(ValueTag::String as u8);
            }
            //
            Self::List(l) => {
                let mut builder = ListBuilder::new_with_offset(buf, buf.len());
                for v in l {
                    builder.push(v);
                }
                builder.finish();
            }
            Self::Record(m) => {
                let mut builder = StreamingRecordBuilder::new_with_offset(buf, buf.len());
                for (k, v) in m {
                    builder.push(*k, v);
                }
                builder.finish();
            }

            Self::Raw(v) => {
                buf.extend(&**v);
            }
        };
    }
}

impl<'a> AsRef<BorrowedValue<'a>> for &BorrowedValue<'a> {
    fn as_ref(&self) -> &BorrowedValue<'a> {
        self
    }
}

impl<'a> From<BTreeMap<&'a [u8], BorrowedValue<'a>>> for BorrowedValue<'a> {
    fn from(val: BTreeMap<&'a [u8], BorrowedValue<'a>>) -> BorrowedValue<'a> {
        BorrowedValue::Record(val)
    }
}

impl<'a, const N: usize, V: Into<BorrowedValue<'a>>> From<[V; N]> for BorrowedValue<'a> {
    fn from(val: [V; N]) -> BorrowedValue<'a> {
        BorrowedValue::List(val.map(Into::into).to_vec())
    }
}

impl<'a, V: Into<BorrowedValue<'a>>> From<Vec<V>> for BorrowedValue<'a> {
    fn from(val: Vec<V>) -> BorrowedValue<'a> {
        BorrowedValue::List(val.into_iter().map(Into::into).collect())
    }
}

impl<'a> From<&'a [u8]> for BorrowedValue<'a> {
    fn from(val: &'a [u8]) -> BorrowedValue<'a> {
        BorrowedValue::Bytes(val)
    }
}

impl<'a> From<&'a str> for BorrowedValue<'a> {
    fn from(val: &'a str) -> BorrowedValue<'a> {
        BorrowedValue::String(val)
    }
}

impl<'a> From<&BorrowedValue<'a>> for BorrowedValue<'a> {
    fn from(value: &BorrowedValue<'a>) -> Self {
        value.clone()
    }
}

macro_rules! impl_from {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl<'a> From<$ty> for BorrowedValue<'a> {
                fn from(val: $ty) -> BorrowedValue<'a> {
                    BorrowedValue::$variant(val)
                }
            }
        )*
    };
}

impl_from!(
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    u128 => U128,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    i128 => I128,
    f32 => F32,
    f64 => F64,
    bool => Boolean,
);
