use afl::fuzz;
use arbitrary::Arbitrary;
use std::{collections::BTreeMap, ops::Deref};
use zcr::{RecordBuilder, RecordKey, ValueAccessor, ValueTag, WrappedValue};

#[derive(Debug, Clone, Arbitrary)]
enum FuzzValue {
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

    Bytes(Vec<u8>),
    String(String),

    List(Vec<FuzzValue>),

    Map(Vec<(String, FuzzValue)>),
}

impl From<FuzzValue> for WrappedValue {
    fn from(v: FuzzValue) -> Self {
        match v {
            FuzzValue::U8(v) => WrappedValue::U8(v),
            FuzzValue::U16(v) => WrappedValue::U16(v),
            FuzzValue::U32(v) => WrappedValue::U32(v),
            FuzzValue::U64(v) => WrappedValue::U64(v),
            FuzzValue::U128(v) => WrappedValue::U128(v),

            FuzzValue::I8(v) => WrappedValue::I8(v),
            FuzzValue::I16(v) => WrappedValue::I16(v),
            FuzzValue::I32(v) => WrappedValue::I32(v),
            FuzzValue::I64(v) => WrappedValue::I64(v),
            FuzzValue::I128(v) => WrappedValue::I128(v),

            FuzzValue::F32(v) => WrappedValue::F32(v),
            FuzzValue::F64(v) => WrappedValue::F64(v),

            FuzzValue::Boolean(v) => WrappedValue::Boolean(v),

            FuzzValue::Bytes(v) => WrappedValue::Bytes(v.into()),

            FuzzValue::String(v) => WrappedValue::String(v),

            FuzzValue::List(v) => WrappedValue::List(v.into_iter().map(Into::into).collect()),

            FuzzValue::Map(entries) => {
                let mut map = BTreeMap::new();

                for (k, v) in entries {
                    map.insert(RecordKey::from(k), v.into());
                }

                WrappedValue::Map(map)
            }
        }
    }
}

fn assert_value_eq(expected: &WrappedValue, actual: ValueAccessor<'_>) {
    match expected {
        WrappedValue::U8(v) => {
            assert_eq!(actual.value_type(), ValueTag::U8);
            assert_eq!(actual.as_u8(), Some(*v));
        }
        WrappedValue::U16(v) => {
            assert_eq!(actual.value_type(), ValueTag::U16);
            assert_eq!(actual.as_u16(), Some(*v));
        }
        WrappedValue::U32(v) => {
            assert_eq!(actual.value_type(), ValueTag::U32);
            assert_eq!(actual.as_u32(), Some(*v));
        }
        WrappedValue::U64(v) => {
            assert_eq!(actual.value_type(), ValueTag::U64);
            assert_eq!(actual.as_u64(), Some(*v));
        }
        WrappedValue::U128(v) => {
            assert_eq!(actual.value_type(), ValueTag::U128);
            assert_eq!(actual.as_u128(), Some(*v));
        }

        WrappedValue::I8(v) => {
            assert_eq!(actual.value_type(), ValueTag::I8);
            assert_eq!(actual.as_i8(), Some(*v));
        }
        WrappedValue::I16(v) => {
            assert_eq!(actual.value_type(), ValueTag::I16);
            assert_eq!(actual.as_i16(), Some(*v));
        }
        WrappedValue::I32(v) => {
            assert_eq!(actual.value_type(), ValueTag::I32);
            assert_eq!(actual.as_i32(), Some(*v));
        }
        WrappedValue::I64(v) => {
            assert_eq!(actual.value_type(), ValueTag::I64);
            assert_eq!(actual.as_i64(), Some(*v));
        }
        WrappedValue::I128(v) => {
            assert_eq!(actual.value_type(), ValueTag::I128);
            assert_eq!(actual.as_i128(), Some(*v));
        }

        WrappedValue::F32(v) => {
            assert_eq!(actual.value_type(), ValueTag::F32);

            // Skip awkward NaNs for now
            if !v.is_normal() {
                return;
            }

            assert_eq!(actual.as_f32(), Some(*v));
        }
        WrappedValue::F64(v) => {
            assert_eq!(actual.value_type(), ValueTag::F64);

            // Skip awkward NaNs for now
            if !v.is_normal() {
                return;
            }

            assert_eq!(actual.as_f64(), Some(*v));
        }

        WrappedValue::Boolean(v) => {
            assert_eq!(actual.value_type(), ValueTag::Boolean);
            assert_eq!(actual.as_bool(), Some(*v));
        }

        WrappedValue::Bytes(v) => {
            assert_eq!(actual.value_type(), ValueTag::Bytes);
            assert_eq!(actual.as_bytes(), Some(v.as_ref()));
        }

        WrappedValue::String(v) => {
            assert_eq!(actual.value_type(), ValueTag::String);
            assert_eq!(actual.as_str(), Some(v.as_str()));
        }

        WrappedValue::List(expected) => {
            let list = actual.as_list().unwrap();

            assert_eq!(list.len(), expected.len());

            for (expected, actual) in expected.iter().zip(list.iter()) {
                assert_value_eq(expected, actual);
            }
        }

        WrappedValue::Map(expected) => {
            let record = actual.as_record().unwrap();

            assert_eq!(record.len(), expected.len());

            for (key, expected_value) in expected {
                let actual = record.get(key).unwrap();
                assert_value_eq(expected_value, actual);
            }
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Input {
    entries: Vec<(String, FuzzValue)>,
}

fn main() {
    fuzz!(|input: Input| {
        let mut expected = BTreeMap::<RecordKey, WrappedValue>::new();
        let mut builder = RecordBuilder::default();

        for (k, v) in &input.entries {
            let value: WrappedValue = v.clone().into();

            expected.insert(RecordKey::from(k.clone()), value.clone());

            builder = builder.prop(k.deref(), value);
        }

        let record = builder.finish();

        //eprintln!("{expected:#?}");
        //eprintln!("{record:#?}");

        assert_eq!(expected.len(), record.len());

        for (k, _) in &input.entries {
            //eprintln!("key={k:?}");
            let expected = expected.get(k.as_bytes()).unwrap();
            let actual = record.get(k.as_bytes()).unwrap();
            assert_value_eq(&expected.clone().into(), actual);
        }
    });
}
