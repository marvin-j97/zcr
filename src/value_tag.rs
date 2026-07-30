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
    // Datetime = 19,
}
