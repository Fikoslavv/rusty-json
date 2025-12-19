mod json_object;
mod primitive_impls;

pub use json_object::JsonObject;

pub fn from_json<T>(json: &str) -> Result<T, String> where T : TryFrom<JsonObject, Error = String>
{
    match T::try_from(JsonObject::try_from(json)?)
    {
        Ok(obj) => Ok(obj),
        Err(msg) => Err(format!("T::try_from(…) failed with message `{}`", msg)),
    }
}

pub fn into_json<T>(obj: T) -> String where T : Into<JsonObject> { obj.into().into_json() }

pub fn to_json<'a, T>(obj: &'a T) -> String where &'a T : Into<JsonObject> { obj.into().into_json() }

#[cfg(feature = "derive")]
pub use rusty_json_derive::{ JsonSerialize, IntoJson, ToJson, JsonDeserialize };
