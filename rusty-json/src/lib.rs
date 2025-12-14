mod json_object;

pub use json_object::JsonObject;

pub fn from_json<T>(json: &str) -> Result<T, &'static str> where T : TryFrom<JsonObject>
{
    match T::try_from(JsonObject::try_from(json)?)
    {
        Ok(obj) => Ok(obj),
        Err(_) => Err("T::try_from(…) failed !"),
    }
}

pub fn into_json<T>(obj: T) -> String where T : Into<JsonObject> { obj.into().into_json() }

pub fn to_json<'a, T>(obj: &'a T) -> String where &'a T : Into<JsonObject> { obj.into().into_json() }
