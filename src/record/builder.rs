use crate::{OwnedRecord, WrappedValue, record::streaming::StreamingRecordBuilder};
use byteview::ByteView;
use std::collections::BTreeMap;

pub type RecordKey = ByteView;

//                               ,__________________
//                ---------------------            |
//               v               v     |           |
// List: [field1][field2][field3][ptr1][ptr2][ptr3][table_start][len][tag]
//       ^               ^       |           |
//       -----------------------´            |
//                       `------------------´
//
//                               ,__________________
//                ---------------------            |
//               v               v     |           |
// Objt: [field1][field2][field3][ptr1][ptr2][ptr3][table_start][len][list tag][key1][key2][key3][keyptr1][keyptr2][keyptr3][keys_start][keyptrstart][tag]
//       ^               ^       |           |
//       -----------------------´            |
//                       `------------------´

/// Record builder, which buffers entries before building, allowing random order insertions
#[derive(Default)]
pub struct RecordBuilder {
    fields: BTreeMap<RecordKey, WrappedValue>,
}

impl RecordBuilder {
    pub fn prop(mut self, key: impl Into<RecordKey>, item: impl Into<WrappedValue>) -> Self {
        self.fields.insert(key.into(), item.into());
        self
    }

    pub fn finish(self) -> OwnedRecord {
        let mut buf = vec![];
        self.finish_into(&mut buf);
        OwnedRecord(buf.into())
    }

    pub fn finish_into(self, buf: &mut Vec<u8>) {
        let mut builder = StreamingRecordBuilder::new_with_offset(buf, buf.len());

        for (k, v) in self.fields {
            builder.push(k, &v);
        }

        builder.finish();
    }
}
