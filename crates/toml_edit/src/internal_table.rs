use crate::{InlineTable, Item, Key, Table, Value};
/// a place for helper methods supporting the table-like impls

use crate::table::KeyValuePairs;

/// GetTableValues provides the logic for displaying a table's items in their parsed order
pub(crate) trait GetTableValues {
    fn items(&self) -> &KeyValuePairs;

    fn get_values(&self) -> Vec<(Vec<&Key>, &Value)> {
        let mut values = Vec::new();
        let root = Vec::new();
        self.append_values(&root, &mut values);
        values
    }

    fn append_values<'s>(
        &'s self,
        parent: &[&'s Key],
        values: &mut Vec<(Vec<&'s Key>, &'s Value)>,
    ) {
        for (key, item) in self.items().iter() {
            let mut path = parent.to_vec();
            path.push(key);
            match item {
                Item::Table(table) if table.is_dotted() => {
                    GetTableValues::append_values(table, &path, values)
                }
                Item::Value(Value::InlineTable(table)) if table.is_dotted() => {
                    GetTableValues::append_values(table, &path, values)
                }
                Item::Value(value) => {
                    values.push((path, value))
                }
                _ => {}
            }
        }
        sort_values_by_position(values);
    }
}

/// SortTable provides the logic for sorting a table's items by its keys
pub(crate) trait SortTable {
    fn items_mut(&mut self) -> &mut KeyValuePairs;

    fn sort_values(&mut self) {
        // Assuming standard tables have their doc_position set and this won't negatively impact them
        self.items_mut().sort_keys();
        assign_sequential_key_positions(self.items_mut(), |item| {
            match item {
                Item::Table(table) if table.is_dotted() => {
                    SortTable::sort_values(table)
                }
                Item::Value(Value::InlineTable(table)) if table.is_dotted() => {
                    SortTable::sort_values(table)
                }
                _ => {}
            }
        });
    }
}

/// SortTableBy provides the logic for sorting a table by a custom comparison
pub(crate) trait SortTableBy<It> : SortTable
where
    It: for<'a> TryFrom<&'a Item>
{
    fn sort_values_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Key, &It, &Key, &It) -> std::cmp::Ordering,
    {
        // intended for `InlineTable`s, where some `Item`s might not be `Value`s,
        // in the case of dotted keys mostly I expect.
        // but for `Table`s the `(Some,Some)` will be the only one used.
        self.sort_vals_by_direct(
            &mut Self::generalize(compare)
        )
    }

    /// no modification to the comparing Fn in this one,
    /// allows for slightly improved recursion that does not continuously
    /// re-modify the comparison function.
    fn sort_vals_by_direct<F>(&mut self, compare: &mut F)
    where
        F: FnMut(&Key, &Item, &Key, &Item) -> std::cmp::Ordering
    {
        self.items_mut().sort_by(|key1, val1, key2, val2| {
            compare(key1, val1, key2, val2)
        });

        assign_sequential_key_positions(self.items_mut(), |value| {
            match value {
                Item::Table(table) if table.is_dotted() => {
                    SortTableBy::<Item>::sort_values_by(
                        table,
                        |k1, i1, k2, i2| {
                            compare(k1, i1, k2, i2)
                        }
                    )
                },
                Item::Value(Value::InlineTable(table)) if table.is_dotted() => {
                    SortTableBy::<Value>::sort_values_by(
                        table,
                        |k1, i1, k2, i2| {
                            let s1 = &Item::from(i1);
                            let s2 = &Item::from(i2);
                            compare(k1, s1, k2, s2)
                        }
                    )
                },
                _ => {}
            };
        });
    }

    fn generalize<'a, F>(mut compare: F) -> Box<dyn FnMut(&Key, &Item, &Key, &Item) -> std::cmp::Ordering + 'a>
    where
        F: FnMut(&Key, &It, &Key, &It) -> std::cmp::Ordering + 'a,
    {
        Box::new(move |key1, s1, key2, s2| {
            match (It::try_from(s1).ok(), It::try_from(s2).ok()) {
                (Some(v1), Some(v2)) => compare(key1, &v1, key2, &v2),
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            }
        })
    }
}

fn assign_sequential_key_positions<F>(items: &mut KeyValuePairs, mut recursive_step: F)
where
    F: FnMut(&mut Item),
{
    use indexmap::map::MutableKeys;
    for (pos, (key, value)) in items.iter_mut2().enumerate() {
        key.set_position(Some(pos));
        recursive_step(value);
    }
}

fn sort_values_by_position<'s>(values: &mut [(Vec<&'s Key>, &'s Value)]) {
    /*
    `Vec::sort_by_key` works because we add the position to _every_ item's key during parsing,
    so keys without positions would be either:
       1. non-leaf keys (i.e. "foo" or "bar" in dotted key "foo.bar.baz")
       2. custom keys added to the doc without an explicit position
    In the case of (1), we'd never see it since we only look at the last
    key in a dotted-key. So, we can safely return a constant value for these cases.

    To support the most intuitive behavior, we return the maximum usize, placing
    position=None items at the end, so when you insert it without position, it
    appends it to the end.
     */
    values.sort_by_key(|(key_path, _)| {
        return match key_path.last().map(|x| x.position) {
            // unwrap "last()" -> unwrap "position"
            Some(Some(pos)) => pos,
            // either last() = None, or position = None
            _ => usize::MAX
        };
    });
}

impl TryFrom<&Item> for Value {
    type Error = String;

    fn try_from(value: &Item) -> Result<Self, Self::Error> {
        let err = "cannot extract Value from Non-Value Item:";
        match value {
            Item::Value(v) => Ok((*v).clone()),
            it => it.as_value().map(|v| v.clone()).ok_or(
                format!("{err}: {it:?}")
            ),
        }
    }

}

impl GetTableValues for Table {
    fn items(&self) -> &KeyValuePairs {
        &self.items
    }
}
impl GetTableValues for InlineTable {
    fn items(&self) -> &KeyValuePairs {
        &self.items
    }
}

impl SortTable for Table {
    fn items_mut(&mut self) -> &mut KeyValuePairs {
        &mut self.items
    }
}
impl SortTable for InlineTable {
    fn items_mut(&mut self) -> &mut KeyValuePairs {
        &mut self.items
    }
}

impl SortTableBy<Item> for Table {}
impl SortTableBy<Value> for InlineTable {}
