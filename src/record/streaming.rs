use crate::{
    ListBuilder, borrowed_value::BorrowedValue, record::builder::RecordKey, value_tag::ValueTag,
};

/// Streaming record builder, which requires sorted insertion of key-value pairs,
/// but only keys need to be buffered in memory
pub struct StreamingRecordBuilder<'a> {
    buf_offset: usize,

    list_builder: ListBuilder<'a>,

    keys: Vec<RecordKey>,
}

impl<'a> StreamingRecordBuilder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        Self::new_with_offset(buf, 0)
    }

    pub(crate) fn new_with_offset(buf: &'a mut Vec<u8>, buf_offset: usize) -> Self {
        Self {
            buf_offset,

            list_builder: ListBuilder::new_with_offset(buf, buf_offset),

            keys: Vec::new(),
        }
    }

    pub fn push<'b>(&mut self, key: impl Into<RecordKey>, value: impl Into<BorrowedValue<'b>>) {
        self.list_builder.push(value);
        self.keys.push(key.into());
    }

    pub fn finish(self) {
        let base_offset = self.buf_offset;

        let (_, buf) = self.list_builder.finish_with_buf();

        // Append keys
        let key_offset = buf.len() - base_offset;

        let mut key_offsets = Vec::with_capacity(self.keys.len());

        for key in self.keys {
            let key_offset = buf.len() - base_offset;

            buf.extend(
                u16::try_from(key.len())
                    .expect("key too large")
                    .to_be_bytes(),
            );
            buf.extend(&*key);

            key_offsets.push(key_offset);
        }

        let key_slots_start = buf.len() - base_offset;

        for key_offset in key_offsets {
            buf.extend(&u32::try_from(key_offset).unwrap().to_be_bytes());
        }

        buf.extend(&u32::try_from(key_offset).unwrap().to_be_bytes());
        buf.extend(&u32::try_from(key_slots_start).unwrap().to_be_bytes());

        buf.push(ValueTag::Record as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BorrowedRecord;

    #[test]
    fn streaming_record_builder() {
        let mut buf = vec![];
        let mut builder = StreamingRecordBuilder::new(&mut buf);

        builder.push("a", "abc");
        builder.push("b", "def");

        builder.finish();

        let record = BorrowedRecord::from_slice(&buf);
        assert_eq!(2, record.len());

        assert_eq!(Some("abc"), record.get(b"a").and_then(|x| x.as_str()));
        assert_eq!(Some("def"), record.get(b"b").and_then(|x| x.as_str()));
    }
}
