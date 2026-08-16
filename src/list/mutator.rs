use crate::{
    BorrowedList, ListBuilder, WrappedValue, borrowed_value::BorrowedValue, list::OwnedList,
};

enum Node<'a> {
    Ref(&'a [u8]),
    Value(WrappedValue),
}

pub struct Mutator<'a> {
    state: Vec<Node<'a>>,
}

impl<'a> Mutator<'a> {
    pub(crate) fn new(list: &'a BorrowedList<'a>) -> Self {
        let mut state = Vec::default();

        for v in list.iter() {
            state.push(Node::Ref(v.inner()));
        }

        Self { state }
    }

    pub fn push(&mut self, value: impl Into<WrappedValue>) {
        self.state.push(Node::Value(value.into()));
    }

    pub fn remove(&mut self, idx: usize) {
        self.state.remove(idx);
    }

    pub fn clear(&mut self) {
        self.state.clear();
    }

    pub fn finish(self) -> OwnedList {
        let mut v = vec![];
        let mut builder = ListBuilder::new(&mut v);

        for v in self.state {
            match v {
                Node::Ref(r) => {
                    builder.push(BorrowedValue::Raw(&*r));
                }
                Node::Value(v) => {
                    builder.push(&v);
                }
            }
        }

        builder.finish();

        OwnedList(v.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BorrowedList, ListBuilder};
    use test_log::test;

    #[test]
    fn list_mutate_identity_smoke_test() {
        let mut v = vec![];

        let l = {
            let mut builder = ListBuilder::new(&mut v);
            builder.push(1u8);
            builder.finish();
            BorrowedList::from_slice(&v)
        };

        let newl = l.mutate(|_| {});

        assert_eq!(l, newl.as_borrowed());
    }

    #[test]
    fn list_mutate_identity() {
        let mut v = vec![];

        let l = {
            let mut builder = ListBuilder::new(&mut v);
            builder.push(1u8);
            builder.push("hello");
            builder.push(true);
            builder.push([1, 2, 3, 4, 5]);
            builder.finish();
            BorrowedList::from_slice(&v)
        };

        let newl = l.mutate(|_| {});

        assert_eq!(l, newl.as_borrowed())
    }

    #[test]
    fn list_mutate_push() {
        let mut v = vec![];

        let l = {
            let mut builder = ListBuilder::new(&mut v);
            builder.push(1u8);
            builder.push("hello");
            builder.push(true);
            builder.push([1, 2, 3, 4, 5]);
            builder.finish();
            BorrowedList::from_slice(&v)
        };

        let newl = l.mutate(|m: &mut Mutator<'_>| {
            m.push("x");
        });

        assert_eq!(Some("x"), newl.last().and_then(|x| x.as_str()));
    }

    #[test]
    fn list_mutate_remove() {
        let mut v = vec![];

        let l = {
            let mut builder = ListBuilder::new(&mut v);
            builder.push(1u8);
            builder.push("hello");
            builder.push(true);
            builder.push([1, 2, 3, 4, 5]);
            builder.finish();
            BorrowedList::from_slice(&v)
        };

        let newl = l.mutate(|m: &mut Mutator<'_>| {
            m.remove(0);
        });

        assert_eq!(Some("hello"), newl.first().and_then(|x| x.as_str()));
    }
}
