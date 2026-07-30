use crate::{
    list::ListBuilder,
    record::{builder::RecordKey, streaming::StreamingRecordBuilder},
    value_tag::ValueTag,
};
use std::collections::BTreeMap;

#[derive(Clone, PartialEq)]
pub enum WrappedValue {
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

    Bytes(byteview::ByteView),
    String(String),

    List(Vec<WrappedValue>),
    Map(BTreeMap<RecordKey, WrappedValue>),
}

impl WrappedValue {
    pub fn write_into(&self, buf: &mut Vec<u8>) {
        match self {
            WrappedValue::U8(v) => {
                buf.push(*v);
                buf.push(ValueTag::U8 as u8);
            }
            WrappedValue::U16(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U16 as u8);
            }
            WrappedValue::U32(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U32 as u8);
            }
            WrappedValue::U64(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U64 as u8);
            }
            WrappedValue::U128(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::U128 as u8);
            }
            //
            WrappedValue::I8(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I8 as u8);
            }
            WrappedValue::I16(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I16 as u8);
            }
            WrappedValue::I32(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I32 as u8);
            }
            WrappedValue::I64(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I64 as u8);
            }
            WrappedValue::I128(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::I128 as u8);
            }
            //
            WrappedValue::F32(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::F32 as u8);
            }
            WrappedValue::F64(v) => {
                buf.extend(v.to_be_bytes());
                buf.push(ValueTag::F64 as u8);
            }
            //
            WrappedValue::Boolean(v) => {
                buf.push(if *v { 1 } else { 0 });
                buf.push(ValueTag::Boolean as u8);
            }
            //
            WrappedValue::Bytes(v) => {
                buf.extend(&**v);
                buf.push(ValueTag::Bytes as u8);
            }
            WrappedValue::String(s) => {
                buf.extend(s.as_bytes());
                buf.push(ValueTag::String as u8);
            }
            //
            WrappedValue::List(l) => {
                let mut builder = ListBuilder::new_with_offset(buf, buf.len());
                for v in l {
                    builder.push(v);
                }
                builder.finish();
            }
            WrappedValue::Map(m) => {
                let mut builder = StreamingRecordBuilder::new_with_offset(buf, buf.len());
                for (key, item) in m {
                    builder.push(key.clone(), item);
                }
                builder.finish();
            }
        };
    }
}

impl<'a> AsRef<WrappedValue> for &WrappedValue {
    fn as_ref(&self) -> &WrappedValue {
        self
    }
}

impl From<BTreeMap<RecordKey, WrappedValue>> for WrappedValue {
    fn from(val: BTreeMap<RecordKey, WrappedValue>) -> WrappedValue {
        WrappedValue::Map(val)
    }
}

impl<'a, const N: usize, V: Into<WrappedValue>> From<[V; N]> for WrappedValue {
    fn from(val: [V; N]) -> WrappedValue {
        WrappedValue::List(val.map(Into::into).to_vec())
    }
}

impl<'a, V: Into<WrappedValue>> From<Vec<V>> for WrappedValue {
    fn from(val: Vec<V>) -> WrappedValue {
        WrappedValue::List(val.into_iter().map(Into::into).collect())
    }
}

impl<'a> From<&'a [u8]> for WrappedValue {
    fn from(val: &'a [u8]) -> WrappedValue {
        byteview::ByteView::new(val).into()
    }
}

impl<'a> From<byteview::ByteView> for WrappedValue {
    fn from(val: byteview::ByteView) -> WrappedValue {
        WrappedValue::Bytes(val)
    }
}

impl<'a> From<&'a str> for WrappedValue {
    fn from(val: &'a str) -> WrappedValue {
        val.to_owned().into()
    }
}

impl<'a> From<String> for WrappedValue {
    fn from(val: String) -> WrappedValue {
        WrappedValue::String(val)
    }
}

impl<'a> From<&WrappedValue> for WrappedValue {
    fn from(value: &WrappedValue) -> Self {
        value.clone()
    }
}

macro_rules! impl_from {
    ($($ty:ty => $v:ident),* $(,)?) => {
        $(
            impl<'a> From<$ty> for WrappedValue {
                fn from(val: $ty) -> WrappedValue {
                    WrappedValue::$v(val)
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
