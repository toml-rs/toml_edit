use std::ops;

use crate::document::Document;
use crate::{InlineTable, Item, Table, Value};

// copied from
// https://github.com/serde-rs/json/blob/master/src/value/index.rs

pub trait Index: crate::private::Sealed {
    #[doc(hidden)]
    fn index<'v>(&self, val: &'v Item) -> Option<&'v Item>;
    #[doc(hidden)]
    fn index_mut<'v>(&self, val: &'v mut Item) -> Option<&'v mut Item>;
}

impl Index for usize {
    fn index<'v>(&self, v: &'v Item) -> Option<&'v Item> {
        match *v {
            Item::ArrayOfTables(ref aot) => aot.values.get(*self),
            Item::Value(ref a) if a.is_array() => a.as_array().and_then(|a| a.values.get(*self)),
            _ => None,
        }
    }
    fn index_mut<'v>(&self, v: &'v mut Item) -> Option<&'v mut Item> {
        match *v {
            Item::ArrayOfTables(ref mut vec) => vec.values.get_mut(*self),
            Item::Value(ref mut a) => a.as_array_mut().and_then(|a| a.values.get_mut(*self)),
            _ => None,
        }
    }
}

impl Index for str {
    fn index<'v>(&self, v: &'v Item) -> Option<&'v Item> {
        match *v {
            Item::Table(ref t) => t.get(self),
            Item::Value(ref v) => v
                .as_inline_table()
                .and_then(|t| t.items.get(self))
                .map(|kv| &kv.value),
            _ => None,
        }
    }
    fn index_mut<'v>(&self, v: &'v mut Item) -> Option<&'v mut Item> {
        match *v {
            Item::Table(ref mut t) => t.get_mut(self),
            Item::Value(ref mut v) => v
                .as_inline_table_mut()
                .and_then(|t| t.items.get_mut(self).map(|kv| &mut kv.value)),
            _ => None,
        }
    }
}

impl Index for String {
    fn index<'v>(&self, v: &'v Item) -> Option<&'v Item> {
        self[..].index(v)
    }
    fn index_mut<'v>(&self, v: &'v mut Item) -> Option<&'v mut Item> {
        self[..].index_mut(v)
    }
}

impl<'a, T: ?Sized> Index for &'a T
where
    T: Index,
{
    fn index<'v>(&self, v: &'v Item) -> Option<&'v Item> {
        (**self).index(v)
    }
    fn index_mut<'v>(&self, v: &'v mut Item) -> Option<&'v mut Item> {
        (**self).index_mut(v)
    }
}

impl<I> ops::Index<I> for Item
where
    I: Index,
{
    type Output = Item;

    fn index(&self, index: I) -> &Item {
        index.index(self).expect("index not found")
    }
}

impl<I> ops::IndexMut<I> for Item
where
    I: Index,
{
    fn index_mut(&mut self, index: I) -> &mut Item {
        index.index_mut(self).expect("index not found")
    }
}

impl<'s> ops::Index<&'s str> for Table {
    type Output = Item;

    fn index(&self, key: &'s str) -> &Item {
        self.get(key).expect("index not found")
    }
}

impl<'s> ops::IndexMut<&'s str> for Table {
    fn index_mut(&mut self, key: &'s str) -> &mut Item {
        self.get_mut(key).expect("index not found")
    }
}

impl<'s> ops::Index<&'s str> for InlineTable {
    type Output = Value;

    fn index(&self, key: &'s str) -> &Value {
        self.get(key).expect("index not found")
    }
}

impl<'s> ops::IndexMut<&'s str> for InlineTable {
    fn index_mut(&mut self, key: &'s str) -> &mut Value {
        self.get_mut(key).expect("index not found")
    }
}

impl<'s> ops::Index<&'s str> for Document {
    type Output = Item;

    fn index(&self, key: &'s str) -> &Item {
        self.root.index(key)
    }
}

impl<'s> ops::IndexMut<&'s str> for Document {
    fn index_mut(&mut self, key: &'s str) -> &mut Item {
        self.root.index_mut(key)
    }
}
