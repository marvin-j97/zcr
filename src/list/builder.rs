use crate::{borrowed_value::BorrowedValue, value_tag::ValueTag};
use byteorder::{BE, WriteBytesExt};

pub type ValueOffset = u32;

/// List builder
pub struct ListBuilder<'a> {
    pub(crate) buf: &'a mut Vec<u8>,
    value_offsets: Vec<(usize, ValueOffset)>,
    buf_offset: usize,
}

impl<'a> ListBuilder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        Self::new_with_offset(buf, 0)
    }

    pub(crate) fn new_with_offset(buf: &'a mut Vec<u8>, buf_offset: usize) -> Self {
        Self {
            buf,
            value_offsets: Vec::new(),
            buf_offset,
        }
    }

    pub fn push<'b>(&mut self, value: impl Into<BorrowedValue<'b>>) {
        let base_offset = self.buf_offset;

        let field_offset = self.buf.len() - base_offset;

        let field_len = {
            let len_before = self.buf.len() - base_offset;
            value.into().write_into(self.buf);
            let len_after = self.buf.len() - base_offset;
            ValueOffset::try_from(len_after - len_before).expect("value too large")
        };

        self.value_offsets.push((field_offset, field_len));
    }

    pub fn finish(self) -> Vec<(usize, ValueOffset)> {
        self.finish_with_buf().0
    }

    pub fn finish_with_buf(self) -> (Vec<(usize, ValueOffset)>, &'a mut Vec<u8>) {
        let offsets_start = self.buf.len() - self.buf_offset;

        for (offset, len) in &self.value_offsets {
            self.buf
                .write_u32::<BE>((*offset).try_into().expect("serialized list too large"))
                .unwrap();

            self.buf.write_u32::<BE>(*len).unwrap();
        }

        self.buf.extend(
            ValueOffset::try_from(offsets_start)
                .expect("offset too large")
                .to_be_bytes(),
        );

        self.buf.extend(
            u32::try_from(self.value_offsets.len())
                .expect("too many keys")
                .to_be_bytes(),
        );

        self.buf.push(ValueTag::List as u8);

        (self.value_offsets, self.buf)
    }
}
