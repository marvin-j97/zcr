//! (Z)ero (C)opy (R)ecord
//!
//! # Example
//!
//! ```
//! use zcr::{BorrowedRecord, record, RecordBuilder};
//!
//! let r = record! {
//!     "name" => "Alice",
//!     "age" => 25u8,
//!     "is_human" => true,
//!     "is_elephant" => false,
//!     "hash" => [1, 2, 3, 4].as_slice(),
//! };
//!
//! let items = r.iter().collect::<Vec<_>>();
//!
//! assert_eq!(
//!     *items,
//!     [
//!         (b"age" as &[u8], r.get(b"age").unwrap()),
//!         (b"hash" as &[u8], r.get(b"hash").unwrap()),
//!         (b"is_elephant" as &[u8], r.get(b"is_elephant").unwrap()),
//!         (b"is_human" as &[u8], r.get(b"is_human").unwrap()),
//!         (b"name" as &[u8], r.get(b"name").unwrap()),
//!     ],
//! );
//!
//! // Load from bytes (e..g a file):
//! //
//! # let bytes = &[] as &[u8];
//! let record = BorrowedRecord::from_slice(bytes);
//! ```

// ## Actual format
//
// 0              n-1
// [...content...][tag]
//
// Integers are serialized in big-endian format:
//
// e.g. U32(4) = [0x0, 0x0, 0x0, 0x4, 0x3]
//
// Strings are simply stored character-by-character.
// The string length can be derived from the slice length.
//
// e.g. String("Hello") = [b"H", b"e", b"l", b"l", b"o", 0x15]
//
// Lists are suffixed by their length and an item slot array.
//
// Maps are lists plus an array of keys and key slots.

mod accessor;
mod borrowed_value;
mod list;
mod record;
mod wrapped_value;

pub use accessor::ValueAccessor;
pub use list::{BorrowedList, ListBuilder, OwnedList};
pub use record::RecordBuilder;
pub use record::{BorrowedRecord, OwnedRecord};

#[doc(hidden)]
pub use wrapped_value::WrappedValue;

#[macro_export]
macro_rules! record {
    (
        $( $key:expr => $value:expr ),* $(,)?
    ) => {
        RecordBuilder::default()
            $(.prop($key, $value))*
            .finish()
    };
}

/// Value types that can be stored in records and lists
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ValueTag {
    Null = 0,

    U8 = 1,
    U16 = 2,
    U32 = 3,
    U64 = 4,
    U128 = 5,

    I8 = 6,
    I16 = 7,
    I32 = 8,
    I64 = 9,
    I128 = 10,

    F32 = 11,
    F64 = 12,

    Boolean = 13,

    Bytes = 14,
    String = 15,

    List = 16,
    Record = 17,
    //
    // #[cfg(feature = "uuid")]
    // Uuid = 16,

    // #[cfg(feature = "date")]
    // Date = 17,
    // #[cfg(feature = "date")]
    // Timestamp = 18,
    // #[cfg(feature = "date")]
    // Datetime = 19,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use test_log::test;

    #[test]
    fn record_macro() {
        let r1 = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("age", 25u8)
            .prop("is_human", true)
            .prop("is_elephant", false)
            .prop("hash", [1, 2, 3, 4].as_slice())
            .finish();

        let r2 = record! {
            "name" => "Alice",
            "age" => 25u8,
            "is_human" => true,
            "is_elephant" => false,
            "hash" => [1, 2, 3, 4].as_slice(),
        };

        assert_eq!(r1, r2);
    }

    #[test]
    fn record_as_values() {
        let r = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("age", 25u8)
            .prop("is_human", true)
            .prop("is_elephant", false)
            .prop("hash", [1, 2, 3, 4].as_slice())
            .finish();

        let items1 = r.as_values().iter().collect::<Vec<_>>();
        let items2 = r.values().collect::<Vec<_>>();

        assert_eq!(items1, items2);
    }

    #[test]
    fn record_iter() {
        let r = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("age", 25u8)
            .prop("is_human", true)
            .prop("is_elephant", false)
            .prop("hash", [1, 2, 3, 4].as_slice())
            .finish();

        let items = r.iter().collect::<Vec<_>>();

        assert_eq!(
            *items,
            [
                (b"age" as &[u8], r.get(b"age").unwrap()),
                (b"hash" as &[u8], r.get(b"hash").unwrap()),
                (b"is_elephant" as &[u8], r.get(b"is_elephant").unwrap()),
                (b"is_human" as &[u8], r.get(b"is_human").unwrap()),
                (b"name" as &[u8], r.get(b"name").unwrap()),
            ],
        );
    }

    #[test]
    fn owned_record_to_list() {
        let r = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("age", 25u8)
            .prop("is_human", true)
            .prop("is_elephant", false)
            .prop("hash", [1, 2, 3, 4].as_slice())
            .finish()
            .into_values();

        assert_eq!(5, r.len());

        {
            let items = r.iter().collect::<Vec<_>>();
            assert_eq!(Some(25), items[0].as_u8());
            assert_eq!(Some([1, 2, 3, 4].as_slice()), items[1].as_bytes());
            assert_eq!(Some(false), items[2].as_bool());
            assert_eq!(Some(true), items[3].as_bool());
            assert_eq!(Some("Alice"), items[4].as_str());
        }
    }

    #[test]
    fn basic() {
        let r = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("age", 25u8)
            .prop("is_human", true)
            .prop("is_elephant", false)
            .prop("hash", [1, 2, 3, 4].as_slice())
            .finish();

        assert_eq!(5, r.len());

        assert!(r.has(b"name"));
        assert!(r.has(b"age"));
        assert!(r.has(b"is_human"));
        assert!(r.has(b"is_elephant"));
        assert!(r.has(b"hash"));
        assert!(!r.has(b"height"));

        assert_eq!(Some(ValueTag::U8), r.get(b"age").map(|x| x.value_type()));
        assert_eq!(
            Some(ValueTag::String),
            r.get(b"name").map(|x| x.value_type())
        );
        assert_eq!(
            Some(ValueTag::Boolean),
            r.get(b"is_human").map(|x| x.value_type()),
        );
        assert_eq!(
            Some(ValueTag::Boolean),
            r.get(b"is_elephant").map(|x| x.value_type()),
        );
        assert_eq!(
            Some(ValueTag::Bytes),
            r.get(b"hash").map(|x| x.value_type())
        );

        assert_eq!(Some("Alice"), r.get(b"name").and_then(|x| x.as_str()));
        assert_eq!(Some(25), r.get(b"age").and_then(|x| x.as_u8()));
        assert_eq!(Some(true), r.get(b"is_human").and_then(|x| x.as_bool()));
        assert_eq!(Some(false), r.get(b"is_elephant").and_then(|x| x.as_bool()));
        assert_eq!(
            Some([1, 2, 3, 4].as_slice()),
            r.get(b"hash").and_then(|x| x.as_bytes()),
        );
    }

    #[test]
    fn ints() {
        {
            let r = RecordBuilder::default().prop("age", 25u8).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::U8), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(25), r.get(b"age").and_then(|x| x.as_u8()));
        }

        {
            let r = RecordBuilder::default().prop("age", 25u16).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::U16), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(25), r.get(b"age").and_then(|x| x.as_u16()));
        }

        {
            let r = RecordBuilder::default().prop("age", 25u32).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::U32), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(25), r.get(b"age").and_then(|x| x.as_u32()));
        }

        {
            let r = RecordBuilder::default().prop("age", 25u64).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::U64), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(25), r.get(b"age").and_then(|x| x.as_u64()));
        }

        {
            let r = RecordBuilder::default().prop("age", 25u128).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::U128), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(25), r.get(b"age").and_then(|x| x.as_u128()));
        }

        {
            let r = RecordBuilder::default().prop("age", -25i8).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::I8), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(-25), r.get(b"age").and_then(|x| x.as_i8()));
        }

        {
            let r = RecordBuilder::default().prop("age", -25i16).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::I16), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(-25), r.get(b"age").and_then(|x| x.as_i16()));
        }

        {
            let r = RecordBuilder::default().prop("age", -25i32).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::I32), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(-25), r.get(b"age").and_then(|x| x.as_i32()));
        }

        {
            let r = RecordBuilder::default().prop("age", -25i64).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::I64), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(-25), r.get(b"age").and_then(|x| x.as_i64()));
        }

        {
            let r = RecordBuilder::default().prop("age", -25i128).finish();
            assert_eq!(1, r.len());
            assert!(r.has(b"age"));
            assert_eq!(Some(ValueTag::I128), r.get(b"age").map(|x| x.value_type()));
            assert_eq!(Some(-25), r.get(b"age").and_then(|x| x.as_i128()));
        }
    }

    #[test]
    fn many_keys() {
        let mut builder = RecordBuilder::default();

        let keys = (0..5_000)
            .into_iter()
            .map(|i| format!("key{i}").into_bytes())
            .collect::<Vec<_>>();

        for k in &keys {
            builder = builder.prop(&**k, 0u8);
        }

        let r = builder.finish();

        assert_eq!(keys.len(), r.len());

        keys.iter().for_each(|k| {
            assert!(r.has(k));
        });
    }

    #[test]
    fn record_with_array() {
        let r = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("digits", [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9])
            .finish();

        assert_eq!(
            [b"digits" as &[u8], b"name"],
            &*r.keys().collect::<Vec<_>>()
        );

        assert_eq!(
            Some(ValueTag::String),
            r.get(b"name").map(|x| x.value_type()),
        );
        assert_eq!(
            Some(ValueTag::List),
            r.get(b"digits").map(|x| x.value_type()),
        );

        assert_eq!(Some("Alice"), r.get(b"name").and_then(|x| x.as_str()));
        assert_eq!(
            Some(10),
            r.get(b"digits").and_then(|d| d.as_list()).map(|l| l.len()),
        );

        let list = r.get(b"digits").and_then(|x| x.as_list()).unwrap();
        assert_eq!(10, list.len());
        assert_eq!(Some(0), list.get(0).and_then(|x| x.as_u8()));
        assert_eq!(Some(1), list.get(1).and_then(|x| x.as_u8()));
        assert_eq!(Some(2), list.get(2).and_then(|x| x.as_u8()));
        assert_eq!(Some(9), list.get(9).and_then(|x| x.as_u8()));
    }

    #[test]
    fn record_with_record_array() {
        let r = RecordBuilder::default()
            .prop(b"name", "Alice")
            .prop(
                b"digit_objects",
                [
                    {
                        let mut map = BTreeMap::default();
                        map.insert(b"value".into(), 0u8.into());
                        map
                    },
                    {
                        let mut map = BTreeMap::default();
                        map.insert(b"value".into(), 1u8.into());
                        map
                    },
                    {
                        let mut map = BTreeMap::default();
                        map.insert(b"value".into(), 2u8.into());
                        map
                    },
                ],
            )
            .finish();

        assert_eq!(
            [b"digit_objects" as &[u8], b"name"],
            &*r.keys().collect::<Vec<_>>()
        );

        assert_eq!(
            Some(ValueTag::String),
            r.get(b"name").map(|x| x.value_type()),
        );
        assert_eq!(
            Some(ValueTag::List),
            r.get(b"digit_objects").map(|x| x.value_type()),
        );

        assert_eq!(Some("Alice"), r.get(b"name").and_then(|x| x.as_str()));
        assert_eq!(Some(3), {
            r.get(b"digit_objects")
                .and_then(|d| d.as_list())
                .map(|l| l.len())
        });

        let list = r.get(b"digit_objects").and_then(|x| x.as_list()).unwrap();
        assert_eq!(3, list.len());
        assert_eq!(
            Some(0),
            list.get(0)
                .and_then(|x| x.as_record())
                .and_then(|r| r.get(b"value"))
                .and_then(|x| x.as_u8()),
        );
        assert_eq!(
            Some(1),
            list.get(1)
                .and_then(|x| x.as_record())
                .and_then(|r| r.get(b"value"))
                .and_then(|x| x.as_u8()),
        );
        assert_eq!(
            Some(2),
            list.get(2)
                .and_then(|x| x.as_record())
                .and_then(|r| r.get(b"value"))
                .and_then(|x| x.as_u8()),
        );
    }

    #[test]
    fn iter_keys() {
        let r = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("age", 25u8)
            .prop("is_human", true)
            .prop("is_elephant", false)
            .finish();

        assert_eq!(
            [b"age" as &[u8], b"is_elephant", b"is_human", b"name"],
            &*r.keys().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn iter_values() {
        let r = RecordBuilder::default()
            .prop("name", "Alice")
            .prop("age", 25u8)
            .prop("is_human", true)
            .prop("is_elephant", false)
            .finish();

        let mut it = r.values();
        assert_eq!(Some(25), it.next().and_then(|x| x.as_u8()));
        assert_eq!(Some(false), it.next().and_then(|x| x.as_bool()));
        assert_eq!(Some(true), it.next().and_then(|x| x.as_bool()));
        assert_eq!(Some("Alice"), it.next().and_then(|x| x.as_str()));
    }

    #[test]
    fn nested_record() {
        let r = RecordBuilder::default()
            .prop(b"name", "Alice")
            .prop(b"langs", {
                let mut map = BTreeMap::default();
                map.insert(b"rust".into(), "good".into());
                map.insert(b"php".into(), ":(".into());
                map.insert(b"fortran".into(), "oh god".into());
                map
            })
            .finish();

        assert_eq!([b"langs" as &[u8], b"name"], &*r.keys().collect::<Vec<_>>());

        assert_eq!(
            Some(ValueTag::Record),
            r.get(b"langs").map(|x| x.value_type()),
        );
        assert_eq!(
            Some(ValueTag::String),
            r.get(b"name").map(|x| x.value_type()),
        );

        assert_eq!(Some("Alice"), r.get(b"name").and_then(|x| x.as_str()));

        {
            let langs = r.get(b"langs").and_then(|langs| langs.as_record()).unwrap();

            assert_eq!(Some("good"), langs.get(b"rust").and_then(|x| x.as_str()));
            assert_eq!(Some(":("), langs.get(b"php").and_then(|x| x.as_str()));
            assert_eq!(
                Some("oh god"),
                langs.get(b"fortran").and_then(|x| x.as_str()),
            );
        }
    }

    #[test]
    fn nested_nested_record() {
        // Shape:
        // {
        //    "name": "Alice",
        //    "country": {
        //        "name": {
        //            "de": "Deutschland",
        //            "en": "Germany",
        //            "is": "Þýskaland",
        //        }
        //    }
        // }
        let r = RecordBuilder::default()
            .prop(b"name", "Alice")
            .prop(b"country", {
                let mut map = BTreeMap::default();
                map.insert(b"name".into(), {
                    let mut map = BTreeMap::default();
                    map.insert(b"de".into(), "Deutschland".into());
                    map.insert(b"en".into(), "Germany".into());
                    map.insert(b"is".into(), "Þýskaland".into());
                    map.into()
                });
                map
            })
            .finish();

        assert_eq!(
            [b"country" as &[u8], b"name"],
            &*r.keys().collect::<Vec<_>>()
        );

        assert_eq!(
            Some(ValueTag::Record),
            r.get(b"country").map(|x| x.value_type()),
        );
        assert_eq!(
            Some(ValueTag::String),
            r.get(b"name").map(|x| x.value_type()),
        );

        assert_eq!(Some("Alice"), r.get(b"name").and_then(|x| x.as_str()));

        {
            let names = r
                .get(b"country")
                .and_then(|country| country.as_record())
                .and_then(|country| country.get(b"name"))
                .and_then(|name| name.as_record())
                .unwrap();

            assert_eq!([b"de", b"en", b"is"], &*names.keys().collect::<Vec<_>>());

            assert_eq!(
                Some("Deutschland"),
                names.get(b"de").and_then(|x| x.as_str()),
            );
            assert_eq!(Some("Germany"), names.get(b"en").and_then(|x| x.as_str()));
            assert_eq!(Some("Þýskaland"), names.get(b"is").and_then(|x| x.as_str()));
        }
    }

    #[test]
    #[ignore]
    fn debug_simple() {
        let r = RecordBuilder::default()
            .prop(b"name", "Alice")
            .prop(b"country", {
                let mut map = BTreeMap::default();
                map.insert(b"name".into(), {
                    let mut map = BTreeMap::default();
                    map.insert(b"de".into(), "Deutschland".into());
                    map.insert(b"en".into(), "Germany".into());
                    map.insert(b"is".into(), "Þýskaland".into());
                    map.into()
                });
                map
            })
            .finish();

        assert_eq!(
            format!("{r:?}"),
            r#"{"country": {"name": {"de": "Deutschland", "en": "Germany", "is": "Þýskaland"}}, "name": "Alice"}"#,
        );
    }

    #[test]
    #[ignore]
    fn debug_pretty() {
        let r = RecordBuilder::default()
            .prop(b"name", "Alice")
            .prop(b"country", {
                // TODO: allow passing in a RecordBuilder to nest objects

                let mut map = BTreeMap::default();
                map.insert(b"name".into(), {
                    let mut map = BTreeMap::default();
                    map.insert(b"de".into(), "Deutschland".into());
                    map.insert(b"en".into(), "Germany".into());
                    map.insert(b"is".into(), "Þýskaland".into());
                    map.into()
                });
                map
            })
            .finish();

        assert_eq!(
            format!("{r:#?}"),
            r#"{
    "country": {
        "name": {
            "de": "Deutschland",
            "en": "Germany",
            "is": "Þýskaland",
        },
    },
    "name": "Alice",
}"#,
        );
    }
}
