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

#[cfg(test)]
mod test
{
    use super::*;

    #[cfg(feature = "derive")]
    mod derive
    {
        use super::*;

        macro_rules! define_serialization_and_deserialization_tests
        {
            ($struct:ty, $test_name_serialization:ident, $test_name_deserialization:ident) =>
            {
                #[test]
                fn $test_name_serialization()
                {
                    let strct: $struct = Default::default();
                    let strct_json = strct.get_expected_json();
                    let strct_serialized_to = to_json(&strct);
                    let strct_serialized_into = into_json(strct);

                    assert_eq!(strct_json, strct_serialized_to, "rusty_json::to_json(…) did not output the predicted json !");
                    assert_eq!(strct_json, strct_serialized_into, "rusty_json::into_json(…) did not output the predicted json !");
                }

                #[test]
                fn $test_name_deserialization()
                {
                    let strct: $struct = Default::default();
                    let strct_deserialized = from_json(&to_json(&strct)).expect(&format!("rusty_json::from_json(…) did not manage to successfully deserialize `{}` !", stringify!($struct)));

                    assert_eq!(strct, strct_deserialized, "Tested struct after serialization and deserialization was not equal to itself from before the process !");
                }
            };
        }

        #[derive(JsonSerialize, JsonDeserialize, Debug, PartialEq, Clone, Copy)]
        struct SomeStruct
        {
            char: char,
            number: usize,
        }

        impl Default for SomeStruct
        {
            fn default() -> Self { Self { char: 'a', number: 2137 } }
        }

        impl SomeStruct
        {
            fn get_expected_json(&self) -> String
            {
                format!( "{{\"char\":\"{}\",\"number\":{}}}", self.char, self.number)
            }
        }

        mod primitives
        {
            use super::*;

            #[derive(JsonSerialize, JsonDeserialize, PartialEq, Debug)]
            struct Primitives
            {
                i8: i8, i16: i16, i32: i32, i64: i64, i128: i128, isize: isize,
                u8: u8, u16: u16, u32: u32, u64: u64, u128: u128, usize: usize,
                f32: f32, f64: f64, bool: bool, char: char, string: String,
            }

            impl Default for Primitives
            {
                fn default() -> Self
                {
                    Self
                    {
                        i8: i8::MIN, i16: i16::MIN, i32: i32::MIN, i64: i64::MIN, i128: i128::MIN, isize: isize::MIN,
                        u8: u8::MAX, u16: u16::MAX, u32: u32::MAX, u64: u64::MAX, u128: u128::MAX, usize: usize::MAX,
                        f32: f32::MAX, f64: f64::MAX, bool: true, char: 'a', string: "test_primitives".to_string()
                    }
                }
            }

            impl Primitives
            {
                fn get_expected_json(&self) -> String
                {
                    format!
                    (
                        "{{\"bool\":\"{}\",\"char\":\"{}\",\"f32\":{},\"f64\":{},\"i128\":{},\"i16\":{},\"i32\":{},\"i64\":{},\"i8\":{},\"isize\":{},\"string\":\"{}\",\"u128\":{},\"u16\":{},\"u32\":{},\"u64\":{},\"u8\":{},\"usize\":{}}}",
                        self.bool, self.char, self.f32, self.f64, self.i128, self.i16, self.i32, self.i64, self.i8, self.isize,self.string, self.u128, self.u16, self.u32, self.u64, self.u8, self.usize
                    )
                }
            }

            define_serialization_and_deserialization_tests!(Primitives, test_derive_primitives_serialization, test_derive_primitives_deserialization);
        }

        mod tuples
        {
            use super::*;

            #[derive(JsonSerialize, JsonDeserialize, Debug, PartialEq)]
            struct IdentTuple
            {
                item: (usize, SomeStruct, char),
                tuple: (char, (char, u8)),
                array: (char, [char; 10]),
            }

            impl Default for IdentTuple
            {
                fn default() -> Self
                {
                    Self
                    {
                        item: (1234, Default::default(), 'f'),
                        tuple: ('A', ('a', 97)),
                        array: ('z', [ '_', 'F', 'i', 'k', 'o', 's', 'ł', 'a', 'w', '_' ])
                    }
                }
            }

            impl IdentTuple
            {
                fn get_expected_json(&self) -> String
                {
                    format!
                    (
                        "{{\"array\":[\"{}\",[{}]],\"item\":[{},{},\"{}\"],\"tuple\":[\"{}\",[\"{}\",{}]]}}",
                        self.array.0, self.array.1.iter().enumerate().map(|(i, c)| format!("\"{}\"{}", c, if i + 1 < self.array.1.len() { "," } else { "" })).collect::<String>(),
                        self.item.0, self.item.1.get_expected_json(), self.item.2, self.tuple.0, self.tuple.1.0, self.tuple.1.1
                    )
                }
            }

            #[derive(JsonSerialize, JsonDeserialize, Debug, PartialEq)]
            struct TupleTuple
            {
                tuple: (i8, (bool, (char, u8))),
                array: (i8, (bool, [char; 10])),
                item: (i8, (bool, SomeStruct)),
            }

            impl Default for TupleTuple
            {
                fn default() -> Self
                {
                    Self
                    {
                        tuple: (-64, (false, ('a', 97))),
                        array: (-16, (true, [ '_', 'F', 'i', 'k', 'o', 's', 'ł', 'a', 'w', '_' ])),
                        item: (-32, (false, Default::default())),
                    }
                }
            }

            impl TupleTuple
            {
                fn get_expected_json(&self) -> String
                {
                    format!
                    (
                        "{{\"array\":[{},[\"{}\",[{}]]],\"item\":[{},[\"{}\",{}]],\"tuple\":[{},[\"{}\",[\"{}\",{}]]]}}",
                        self.array.0, self.array.1.0, self.array.1.1.iter().enumerate().map(|(i, c)| format!("\"{}\"{}", c, if i + 1 < self.array.1.1.len() { "," } else { "" })).collect::<String>(),
                        self.item.0, self.item.1.0, self.item.1.1.get_expected_json(), self.tuple.0, self.tuple.1.0, self.tuple.1.1.0, self.tuple.1.1.1
                    )
                }
            }

            #[derive(JsonSerialize, JsonDeserialize, Debug, PartialEq)]
            struct ArrayTuple
            {
                array: [(SomeStruct, [bool; 2]); 1],
                tuple: [(SomeStruct, (char, bool)); 1],
                item: (SomeStruct, [bool; 2]),
            }

            impl Default for ArrayTuple
            {
                fn default() -> Self
                {
                    Self
                    {
                        array: [ (Default::default(), [ false, true ]) ],
                        tuple: [ (Default::default(), ('a', true)) ],
                        item: (Default::default(), [ false, true ])
                    }
                }
            }

            impl ArrayTuple
            {
                fn get_expected_json(&self) -> String
                {
                    format!
                    (
                        "{{\"array\":[{}],\"item\":[{},[{}]],\"tuple\":[{}]}}",
                        self.array.iter().map(|(s, a)| format!("[{},[{}]]", s.get_expected_json(), a.iter().enumerate().map(|(i, b)| format!("\"{b}\"{}", if i + 1 < a.len() { "," } else { "" })).collect::<String>())).collect::<String>(),
                        self.item.0.get_expected_json(), self.item.1.iter().enumerate().map(|(i, b)| format!("\"{b}\"{}", if i + 1 < self.item.1.len() { "," } else { "" })).collect::<String>(),
                        self.tuple.iter().map(|(s, (c, b))| format!("[{},[\"{c}\",\"{b}\"]]", s.get_expected_json())).collect::<String>()
                    )
                }
            }

            define_serialization_and_deserialization_tests!(IdentTuple, test_derive_tuple_ident_serialization, test_derive_tuple_ident_deserialization);

            define_serialization_and_deserialization_tests!(TupleTuple, test_derive_tuple_tuple_serialization, test_derive_tuple_tuple_deserialization);

            define_serialization_and_deserialization_tests!(ArrayTuple, test_derive_tuple_array_serialization, test_derive_tuple_array_deserialization);
        }

        mod arrays
        {
            use super::*;

            #[derive(JsonSerialize, JsonDeserialize, Debug, PartialEq)]
            struct Array
            {
                ident: [char; 10],
                array: [[u8; 2]; 2],
                tuple: (SomeStruct, [bool; 3]),
            }

            impl Default for Array
            {
                fn default() -> Self
                {
                    Self
                    {
                        ident: [ '_', 'F', 'i', 'k', 'o', 's', 'ł', 'a', 'w', '_' ],
                        array: [ [ 13, 37 ], [ 21, 37 ] ],
                        tuple: (Default::default(), [ true, true, false ]),
                    }
                }
            }

            impl Array
            {
                fn get_expected_json(&self) -> String
                {
                    format!
                    (
                        "{{\"array\":[{}],\"ident\":[{}],\"tuple\":[{},[{}]]}}",
                        self.array.iter().enumerate().map(|(i, a)| format!("[{}]{}", a.iter().enumerate().map(|(i, b)| b.to_string() + if i + 1 < a.len() { "," } else { "" }).collect::<String>(), if i + 1 < self.array.len() { "," } else { "" })).collect::<String>(),
                        self.ident.iter().enumerate().map(|(i, c)| format!("\"{c}\"{}", if i + 1 < self.ident.len() { "," } else { "" })).collect::<String>(),
                        self.tuple.0.get_expected_json(), self.tuple.1.iter().enumerate().map(|(i, b)| format!("\"{b}\"{}", if i + 1 < self.tuple.1.len() { "," } else { "" })).collect::<String>()
                    )
                }
            }

            define_serialization_and_deserialization_tests!(Array, test_derive_array_serialization, test_derive_array_deserialization);
        }
    }
}
