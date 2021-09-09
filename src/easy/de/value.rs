use crate::easy::de::Error;

pub(crate) struct ValueDeserializer {
    input: crate::Value,
}

impl ValueDeserializer {
    pub(crate) fn new(input: crate::Value) -> Self {
        Self { input }
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut input: crate::Value = 0.into();
        std::mem::swap(&mut input, &mut self.input);
        match input {
            crate::Value::String(v) => visitor.visit_string(v.into_value()),
            crate::Value::Integer(v) => visitor.visit_i64(v.into_value()),
            crate::Value::Float(v) => visitor.visit_f64(v.into_value()),
            crate::Value::Boolean(v) => visitor.visit_bool(v.into_value()),
            crate::Value::OffsetDateTime(v) => visitor.visit_string(v.into_value().to_string()),
            crate::Value::LocalDateTime(v) => visitor.visit_string(v.into_value().to_string()),
            crate::Value::LocalDate(v) => visitor.visit_string(v.into_value().to_string()),
            crate::Value::LocalTime(v) => visitor.visit_string(v.into_value().to_string()),
            crate::Value::Array(v) => {
                visitor.visit_seq(crate::easy::de::ArraySeqAccess::with_array(v))
            }
            crate::Value::InlineTable(v) => {
                visitor.visit_map(crate::easy::de::InlineTableMapAccess::new(v))
            }
        }
    }

    // `None` is interpreted as a missing field so be sure to implement `Some`
    // as a present field.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map unit newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier struct
    }
}
