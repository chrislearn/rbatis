use crate::value::ext::Error;
use crate::ValueRef;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};

#[derive(Clone)]
pub struct SerRef {}

pub struct SerializeStructImpl<'a> {
    s: SerRef,
    inner: Vec<(ValueRef<'a>, ValueRef<'a>)>,
}

impl<'a> SerializeStruct for SerializeStructImpl<'a> {
    type Ok = ValueRef<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner
            .push((ValueRef::String(key), value.serialize(self.s.clone())?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Map(self.inner))
    }
}

pub struct SerializeSeqImpl<'a> {
    s: SerRef,
    inner: Vec<ValueRef<'a>>,
}

impl<'a> SerializeSeq for SerializeSeqImpl<'a> {
    type Ok = ValueRef<'a>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(self.s.clone())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Array(self.inner))
    }
}

pub struct SerializeTupleImpl<'a> {
    s: SerRef,
    inner: Vec<ValueRef<'a>>,
}

impl<'a> SerializeTuple for SerializeTupleImpl<'a> {
    type Ok = ValueRef<'a>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(self.s.clone())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Array(self.inner))
    }
}

pub struct SerializeTupleStructImpl<'a> {
    s: SerRef,
    inner: Vec<ValueRef<'a>>,
}

impl<'a> SerializeTupleStruct for SerializeTupleStructImpl<'a> {
    type Ok = ValueRef<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(self.s.clone())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Array(self.inner))
    }
}

pub struct SerializeTupleVariantImpl<'a> {
    s: SerRef,
    inner: Vec<ValueRef<'a>>,
}

impl<'a> SerializeTupleVariant for SerializeTupleVariantImpl<'a> {
    type Ok = ValueRef<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(self.s.clone())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Array(self.inner))
    }
}

pub struct SerializeMapImpl<'a> {
    s: SerRef,
    inner: Vec<(ValueRef<'a>, ValueRef<'a>)>,
}

impl<'a> SerializeMap for SerializeMapImpl<'a> {
    type Ok = ValueRef<'a>;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner
            .push((key.serialize(self.s.clone())?, ValueRef::Null));
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self.inner.last_mut() {
            None => {}
            Some((_, v)) => {
                *v = value.serialize(self.s.clone())?;
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Map(self.inner))
    }
}

pub struct SerializeStructVariantImpl<'a> {
    s: SerRef,
    inner: Vec<ValueRef<'a>>,
}

impl<'a> SerializeStructVariant for SerializeStructVariantImpl<'a> {
    type Ok = ValueRef<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(self.s.clone())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Array(self.inner))
    }
}

impl Serializer for SerRef {
    type Ok = ValueRef<'static>;
    type Error = Error;
    type SerializeSeq = SerializeSeqImpl<'static>;
    type SerializeTuple = SerializeTupleImpl<'static>;
    type SerializeTupleStruct = SerializeTupleStructImpl<'static>;
    type SerializeTupleVariant = SerializeTupleVariantImpl<'static>;
    type SerializeMap = SerializeMapImpl<'static>;
    type SerializeStruct = SerializeStructImpl<'static>;
    type SerializeStructVariant = SerializeStructVariantImpl<'static>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::I32(v as i32))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::I32(v as i32))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::I64(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::U32(v as u32))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::U32(v as u32))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::U64(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::I64(v as i64))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::String(unsafe { &*(v as *const str) }.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Binary(unsafe { &*(v as *const [u8]) }.into()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Null)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::Null)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(ValueRef::String(variant.into()))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(ValueRef::Ext(name, Box::new(value.serialize(self)?)))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(Self::SerializeSeq {
            s: self.clone(),
            inner: Vec::with_capacity(len.unwrap_or_default()),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Self::SerializeTuple {
            s: self.clone(),
            inner: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Self::SerializeTupleStruct {
            s: self.clone(),
            inner: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(Self::SerializeTupleVariant {
            s: self.clone(),
            inner: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(Self::SerializeMap {
            s: self.clone(),
            inner: Vec::with_capacity(len.unwrap_or_default()),
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Self::SerializeStruct {
            s: self.clone(),
            inner: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(Self::SerializeStructVariant {
            s: self.clone(),
            inner: Vec::with_capacity(len),
        })
    }
}

/// serialize an value ref
pub fn to_value_ref<T>(a: &T) -> Result<ValueRef, Error>
where
    T: serde::Serialize,
{
    a.serialize(SerRef {})
}

impl<'a> ValueRef<'a> {
    /// serialize an value ref
    pub fn serialize<T: Serialize>(&mut self, arg: &'a T) -> Result<(), Error> {
        *self = to_value_ref(arg)?;
        Ok(())
    }
}
