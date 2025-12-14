use crate::JsonObject;

macro_rules! impl_deserialize_for_primitive
{
    (String) =>
    {
        impl TryFrom<JsonObject> for String
        {
            type Error = String;

            fn try_from(json: JsonObject) -> Result<Self, Self::Error>
            {
                println!("This is a string version of the macro !");

                match json
                {
                    JsonObject::Value { value } => Ok(value),
                    obj => return Err(format!("Cannot deserialize {} from `{}`", stringify!($typ), obj)),
                }
            }
        }
    };

    ($typ:ty) =>
    {
        impl TryFrom<JsonObject> for $typ
        {
            type Error = String;

            fn try_from(json: JsonObject) -> Result<Self, Self::Error>
            {
                let string = match json
                {
                    JsonObject::Value { value } => value,
                    obj => return Err(format!("Cannot deserialize {} from `{}`", stringify!($typ), obj)),
                };

                match string.parse::<$typ>()
                {
                    Ok(parsed) => Ok(parsed),
                    Err(_) => Err(format!("Cannot parse `{}` into {} !", string, stringify!($typ))),
                }
            }
        }
    };
}

macro_rules! impl_serialize_for_primitive
{
    ($typ:ty) =>
    {
        impl Into<JsonObject> for $typ
        {
            fn into(self) -> JsonObject
            {
                (&self).into()
            }
        }

        impl Into<JsonObject> for &$typ
        {
            fn into(self) -> JsonObject
            {
                JsonObject::Value { value: self.to_string() }
            }
        }
    };
}

impl_serialize_for_primitive!(i8);
impl_deserialize_for_primitive!(i8);
impl_serialize_for_primitive!(i16);
impl_deserialize_for_primitive!(i16);
impl_serialize_for_primitive!(i32);
impl_deserialize_for_primitive!(i32);
impl_serialize_for_primitive!(i64);
impl_deserialize_for_primitive!(i64);
impl_serialize_for_primitive!(i128);
impl_deserialize_for_primitive!(i128);
impl_serialize_for_primitive!(isize);
impl_deserialize_for_primitive!(isize);

impl_serialize_for_primitive!(u8);
impl_deserialize_for_primitive!(u8);
impl_serialize_for_primitive!(u16);
impl_deserialize_for_primitive!(u16);
impl_serialize_for_primitive!(u32);
impl_deserialize_for_primitive!(u32);
impl_serialize_for_primitive!(u64);
impl_deserialize_for_primitive!(u64);
impl_serialize_for_primitive!(u128);
impl_deserialize_for_primitive!(u128);
impl_serialize_for_primitive!(usize);
impl_deserialize_for_primitive!(usize);

impl_serialize_for_primitive!(f32);
impl_deserialize_for_primitive!(f32);
impl_serialize_for_primitive!(f64);
impl_deserialize_for_primitive!(f64);

impl_serialize_for_primitive!(char);
impl_deserialize_for_primitive!(char);

impl_serialize_for_primitive!(bool);
impl_deserialize_for_primitive!(bool);

impl_serialize_for_primitive!(String);
impl_deserialize_for_primitive!(String);

/* impl <T> Into<JsonObject> for Vec<T> where T : Into<JsonObject>
{
    fn into(self) -> JsonObject
    {
        JsonObject::Array { array: self.into_iter().map(|x| x.into()).collect() }
    }
}

impl <'a, T> Into<JsonObject> for &'a Vec<T> where &'a T : Into<JsonObject>
{
    fn into(self) -> JsonObject
    {
        JsonObject::Array { array: self.into_iter().map(|x| x.into()).collect() }
    }
}

impl <T> TryFrom<JsonObject> for Vec<T> where T : TryFrom<JsonObject>
{
    type Error = String;

    fn try_from(json: JsonObject) -> Result<Self, Self::Error>
    {
        let array = match json
        {
            JsonObject::Array { array } => array,
            obj => return Err(format!("Cannot deserialize Vec<_> from `{}` !", obj)),
        };

        Ok(array.into_iter().filter_map(|x| x.try_into().ok()).collect())
    }
} */
