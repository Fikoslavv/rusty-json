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

#[cfg(test)]
mod tests
{
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn json_test_deserialization_from_str_to_json_obj()
    {
        let json_str = "[ null, 'null', { num : 2137, str : \"1337\", arr : [ 'sth', null ], empty_obj : {}, empty_arr : [] } ]";

        let obj = from_json(json_str).unwrap();

        let mut iter = match obj
        {
            JsonObject::Array { array } if array.len() == 3 => array.into_iter(),
            JsonObject::Array { array } => panic!("An array containing three elements was expected while the array contains {} element(s) !", array.len()),
            obj => panic!("An array was expected !  The array was deserialized to `{}` !", obj),
        };

        match iter.next().expect("[TEST_PANIC] The iterator over the root array was shorter than expected !")
        {
            JsonObject::Null => (),
            obj => panic!("Null was expected while the object was `{}`", obj),
        }

        match iter.next().expect("[TEST_PANIC] The iterator over the root array was shorter than expected !")
        {
            JsonObject::Value { value } if value == "null" => (),
            JsonObject::Value { value } => panic!("Value of `\"null\"` was expected while the value contained `\"{}\"` !", value),
            obj => panic!("Value `\"null\"` was expected while the object was `{}` !", obj)
        }

        let obj_last = match iter.next().expect("[TEST_PANIC] The iterator over the root array was shorter than expected !")
        {
            JsonObject::Object { fields } if fields.len() == 5 => fields,
            JsonObject::Object { fields } => panic!("Object containing five fields was expected while the object contained {} field(s) !", fields.len()),
            obj => panic!("Object was expected while the object was `{}` !", obj)
        };

        fn test_object_field<'a>(fields: &'a BTreeMap<String, JsonObject>, field_name: &str, expected_field_value: JsonObject) -> &'a JsonObject
        {
            let field = match fields.get(field_name)
            {
                Some(field) => field,
                None => panic!("Object was supposed to have field `{}` !", field_name),
            };

            match field
            {
                obj if obj == &expected_field_value => obj,
                obj => panic!("The field was supposed to be of type `{}` while it was of type `{}` !", expected_field_value, obj)
            }
        }

        test_object_field(&obj_last, "num", JsonObject::Value { value: "2137".to_string() });
        test_object_field(&obj_last, "str", JsonObject::Value { value: "1337".to_string() });
        test_object_field(&obj_last, "arr", JsonObject::Array { array: vec![ JsonObject::Value { value: "sth".to_string() }, JsonObject::Null ] });
        test_object_field(&obj_last, "empty_obj", JsonObject::Object { fields: BTreeMap::new() });
        test_object_field(&obj_last, "empty_arr", JsonObject::Array { array: Vec::new() });
    }
}
