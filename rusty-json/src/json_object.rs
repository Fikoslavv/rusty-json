use std::{collections::BTreeMap, fmt::Display};

#[derive(Debug, PartialEq)]
pub enum JsonObject
{
    Null,
    Object { fields: BTreeMap<String, JsonObject> },
    Value { value: String },
    Array { array: Vec<JsonObject> },
}

impl Display for JsonObject
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let str = match self
        {
            JsonObject::Null => stringify!(JsonObject::Null),
            JsonObject::Object { fields: _ } => stringify!(JsonObject::Object),
            JsonObject::Value { value: _ } => stringify!(JsonObject::Value),
            JsonObject::Array { array: _ } => stringify!(JsonObject::Array),
        };

        write!(formatter, "{}", str)
    }
}

impl TryFrom<&str> for JsonObject
{
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error>
    {
        let value = value.trim();
        // println!("Deserializing => `{}`", value);

        match value.chars().count()
        {
            0 => return Ok(Self::Null),
            1 => return Err("Given string is not valid json !"),
            _ => { }
        }

        let value = &value.chars().collect::<Vec<char>>()[..];

        match value.first().unwrap()
        {
            '{' => deserialize_object(value),
            '[' => deserialize_array(value),
            _ => deserialize_string(value),
        }
    }
}

impl JsonObject
{
    pub fn into_json(self) -> String { Self::to_json(&self) }

    pub fn to_json(&self) -> String
    {
        match self
        {
            JsonObject::Null => "null".to_string(),
            JsonObject::Value { value } =>
            {
                if value.parse::<i128>().is_ok() || value.parse::<u128>().is_ok() || value.parse::<f64>().is_ok()
                {
                    value.clone()
                }
                else
                {
                    serialize_string(&value)
                }
            }
            JsonObject::Object { fields } =>
            {
                if fields.is_empty() { return "{}".to_string(); }

                let mut serialized_fields: Vec<String> = Vec::with_capacity(fields.len());

                for (field_name, field_value) in fields.into_iter()
                {
                    let field_name = serialize_string(field_name);
                    let field_value = Self::to_json(field_value);
                    let mut serialized_field = String::with_capacity(1 + field_name.len() + field_value.len());
                    serialized_field.push_str(&field_name);
                    serialized_field.push(':');
                    serialized_field.push_str(&field_value);

                    serialized_fields.push(serialized_field);
                }

                construct_object_or_array(serialized_fields, '{', ',', '}')
            }
            JsonObject::Array { array: objects } =>
            {
                if objects.is_empty() { return "[]".to_string(); }

                let mut serialized_objects: Vec<String> = Vec::with_capacity(objects.len());
                for object in objects
                {
                    serialized_objects.push(Self::to_json(object));
                }

                construct_object_or_array(serialized_objects, '[', ',', ']')
            }
        }
    }
}

fn deserialize_object(value: &[char]) -> Result<JsonObject, &'static str>
{
    let value = value.trim();
    // println!("Deserializing object => `{}`", value.iter().collect::<String>());

    if value.is_empty() || !(*value.first().unwrap() == '{' && *value.last().unwrap() == '}') { return Err("Given string is not valid json !"); }
    let value = (&value[1..value.len() - 1]).trim();

    if value.len() == 0 { return Ok(JsonObject::Object { fields: BTreeMap::new() }); }

    let comma_index = find_chars(value, ',', false)?;

    // let mut object_fields: HashMap<String, JsonObject> = HashMap::with_capacity(comma_index.len() + 1);
    let mut object_fields: BTreeMap<String, JsonObject> = BTreeMap::new();
    if comma_index.is_empty()
    {
        let (key, value) = deserialize_field(value)?;
        object_fields.insert(key, value);
    }
    else
    {
        let mut previous_index: usize = *comma_index.first().unwrap();

        {
            let (key, value) = deserialize_field(&value[0..previous_index])?;
            object_fields.insert(key, value);
        }
        for comma_index in comma_index.into_iter().skip(1)
        {
            let (key, value) = deserialize_field(&value[(previous_index + 1)..comma_index])?;
            object_fields.insert(key, value);
            previous_index = comma_index;
        }
        if previous_index < value.len() - 1
        {
            let (key, value) = deserialize_field(&value[(previous_index + 1)..])?;
            object_fields.insert(key, value);
        }
    }

    Ok(JsonObject::Object { fields: object_fields })
}

fn deserialize_array(value: &[char]) -> Result<JsonObject, &'static str>
{
    let value = value.trim();
    // println!("Deserializing array => `{}`", value.iter().collect::<String>());

    if value.is_empty() || !(*value.first().unwrap() == '[' && *value.last().unwrap() == ']') { return Err("Given string is not valid json !"); }
    let value = (&value[1..value.len() - 1]).trim();
    if value.is_empty() { return Ok(JsonObject::Array { array: Vec::new() }); }

    let mut comma_indexes = find_chars(value, ',', false)?;

    if comma_indexes.is_empty()
    {
        let obj = match value.first().unwrap()
        {
            '[' => deserialize_array(value)?,
            '{' => deserialize_object(value)?,
            _ => deserialize_string(value)?,
        };

        Ok(JsonObject::Array { array: vec![ obj ] })
    }
    else
    {
        if *comma_indexes.last().unwrap() != value.len() - 1 { comma_indexes.push(value.len()); }

        let mut objects: Vec<JsonObject> = Vec::with_capacity(comma_indexes.len() + 1);

        let mut previous_index = usize::MAX;
        for comma_index in comma_indexes.into_iter()
        {
            let decodee = (&value[previous_index.wrapping_add(1)..comma_index]).trim();
            previous_index = comma_index;

            match decodee.first().unwrap()
            {
                '[' => objects.push(deserialize_array(decodee)?),
                '{' => objects.push(deserialize_object(decodee)?),
                _ => objects.push(deserialize_string(decodee)?),
            };
        }

        Ok(JsonObject::Array { array: objects })
    }
}

fn deserialize_field(value: &[char]) -> Result<(String, JsonObject), &'static str>
{
    let value = value.trim();
    // println!("Deserializing field => `{}`", value.iter().collect::<String>());

    let colon_index = find_chars(value, ':', true)?;
    let colon_index = colon_index.first();

    let colon_index = if let Some(colon_index) = colon_index { *colon_index } else { return Err("Given string is not valid json !"); };

    let field_name = match deserialize_string(&value[..colon_index])?
    {
        JsonObject::Null => format!("null"),
        JsonObject::Value { value } => value,
        _ => return Err("JsonObject::decode_string(…) returned unexpected JsonObject type !"),
    };

    let field_value_str = (&value[(colon_index + 1)..]).trim();
    let field_value =
    match field_value_str.first().unwrap()
    {
        '[' => deserialize_array(field_value_str)?,
        '{' => deserialize_object(field_value_str)?,
        _ => deserialize_string(field_value_str)?,
    };

    Ok((field_name, field_value))
}

fn deserialize_string(value: &[char]) -> Result<JsonObject, &'static str>
{
    let value = value.trim();
    if value.is_empty() { return Err("An empty string cannot be deserialized !") }
    else if value.iter().eq([ 'n', 'u', 'l', 'l' ].iter()) { return Ok(JsonObject::Null) }

    let string_closing_char =
    {
        let first_char = *value.first().unwrap();
        if first_char == '"' || first_char == '\''
        {
            if value.last().unwrap() == &first_char { Some(first_char) }
            else { return Err("A string must start and end with the same quotation mark (either single or double quotes) !") }
        }
        else { None }
    };
    let mut deserialized = String::with_capacity(value.len());
    let mut it = (if string_closing_char.is_some() { &value[1..value.len() - 1] } else { &value }).into_iter();
    let mut is_escaping = false;
    while let Some(char) = it.next()
    {
        match char
        {
            _ if char.is_control() => return Err("Control characters in strings must be represented as `\\uXXXX` where X is a hex number !"),
            '\\' if !is_escaping => is_escaping = true,
            'u' if is_escaping =>
            {
                is_escaping = false;
                if let Ok(unescaped) = u32::from_str_radix(it.by_ref().take(4).collect::<String>().as_str(), 16)
                {
                    if let Some(char_unescaped) = char::from_u32(unescaped) { deserialized.push(char_unescaped) }
                    else { return Err("Given string contains an escape with value out of range for utf-8 encoding !") }
                }
                else { return Err("Given string contains an escape with codepoint unparsable as a hex number !") }
            },
            _ if is_escaping =>
            {
                match char
                {
                    '\\' | '"' | '\'' | '/' | '\u{8}' | '\u{c}' | '\n' | '\r' | '\t' => (),
                    _ => return Err("Given string contains an invalid escape !"),
                }

                is_escaping = false;
                deserialized.push(*char);
            },
            '"' | '\'' if string_closing_char.is_none() || (string_closing_char.is_some() && string_closing_char.unwrap() == *char) => return Err("Given string contains unescaped quotation marks !"),
            ' ' if string_closing_char.is_none() => return Err("Unquoted string may only be one word !"),
            _ => deserialized.push(*char),
        }
    }

    deserialized.shrink_to_fit();
    Ok(JsonObject::Value { value: deserialized })
}

fn find_chars(value: &[char], char_to_find: char, break_on_first_occurence: bool) -> Result<Vec<usize>, &'static str>
{
    let mut string_closing_char: Option<char> = None;
    let mut is_escaping = false;
    let mut object_level = 0u64;
    let mut array_level = 0u64;
    let mut comma_index: Vec<usize> = Vec::with_capacity(value.len());
    for (index, char) in value.iter().enumerate()
    {
        match char
        {
            '"' | '\'' if !is_escaping =>
            {
                if string_closing_char.is_none()
                {
                    if let Some(closing_char) = string_closing_char
                    {
                        if *char == closing_char { string_closing_char = None; }
                    }
                    else { string_closing_char = Some(*char); }
                }
                else { string_closing_char = None; }
            },
            '{' => if string_closing_char.is_none() { object_level += 1; },
            '[' => if string_closing_char.is_none() { array_level += 1; },
            '}' if string_closing_char.is_none() && !is_escaping =>
            {
                if object_level > 0 { object_level -= 1; }
                else { return Err("Given string is not valid json !"); }
            },
            ']' if string_closing_char.is_none() =>
            {
                if array_level > 0 { array_level -= 1; }
                else { return Err("Given string is not valid json !"); }
            },
            '\\' if string_closing_char.is_some() => is_escaping = !is_escaping,
            char if *char == char_to_find && string_closing_char.is_none() && object_level == 0 && array_level == 0 =>
            {
                comma_index.push(index);

                if break_on_first_occurence { break; }
            },
            _ => is_escaping = false,
        }
    };

    comma_index.shrink_to_fit();
    Ok(comma_index)
}

fn serialize_string(value: &str) -> String
{
    let mut serialized = String::with_capacity(value.len() + 2);
    let value = value.chars();
    serialized.push('"');

    for char in value.into_iter()
    {
        match char
        {
            '\\' => serialized.push_str(r"\\"),
            '"' => serialized.push_str("\\\""),
            _ if char.is_ascii_control() =>
            {
                serialized.try_reserve(5).unwrap();
                serialized.push_str("\\u");
                let escaped = char.escape_unicode();
                let escaped_len = escaped.len();
                let escaped_value_len = escaped_len - 4;
                for _ in 0..(4 - escaped_value_len) { serialized.push('0'); }
                for c in escaped.skip(3).take(escaped_value_len) { serialized.push(c); }
            },
            _ => serialized.push(char),
        };
    }

    serialized.push('"');
    serialized
}

fn construct_object_or_array(strings: Vec<String>, open_char: char, separator_char: char, close_char: char) -> String
{
    let mut serialized = String::with_capacity(1 + strings.iter().map(|s| s.len() + 1).sum::<usize>());
    serialized.push(open_char);

    for object in &strings[..strings.len() - 1]
    {
        serialized.push_str(&object);
        serialized.push(separator_char);
    }

    serialized.push_str(strings.last().unwrap());
    serialized.push(close_char);

    serialized
}

trait Trimmable<T>
{
    fn trim(&self) -> T;
}

impl <'a> Trimmable<&'a[char]> for &'a [char]
{
    fn trim(&self) -> &'a [char]
    {
        let mut begin: usize = 0;
        let mut end: usize = 0;

        for (index, char) in self.iter().enumerate()
        {
            begin = index;

            if !char.is_whitespace() { break; }
        }

        for (index, char) in self.iter().rev().enumerate()
        {
            end = index;

            if !char.is_whitespace() { break; }
        }

        &self[begin..(self.len() - end)]
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    mod deserializer
    {
        use super::*;

        macro_rules! static_string_to_char_slice
        {
            ($input:literal) =>
            {
                &$input.chars().collect::<Vec<char>>()
            };
        }

        mod string
        {
            use super::*;

            #[test]
            fn test_deserialize_string_unquoted_empty()
            {
                match deserialize_string(&Vec::new())
                {
                    Err(_) => (),
                    _ => panic!("deserialize_string returned Ok!"),
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_empty()
            {
                match deserialize_string(static_string_to_char_slice!(r#""""#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value.len() == 0 { return }
                            else { panic!("deserialize_string returned JsonObject::Value with length > 0!") }
                        }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_empty()
            {
                match deserialize_string(static_string_to_char_slice!(r#"''"#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value.len() == 0 { return }
                            else { panic!("deserialize_string returned JsonObject::Value with length > 0!") }
                        }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_invalid_ends_with_double_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#"something""#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_invalid_ends_with_single_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#"something'"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_invalid_starts_with_double_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#""something"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_invalid_starts_with_single_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'something"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_invalid_starts_with_double_quote_ends_with_single_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#""something'"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_invalid_starts_with_single_quote_ends_with_double_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'something""#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_invalid_with_leading_word()
            {
                match deserialize_string(static_string_to_char_slice!(r#"hello"world""#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_invalid_with_leading_word()
            {
                match deserialize_string(static_string_to_char_slice!(r#"hello'world'"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_invalid_with_trailing_word()
            {
                match deserialize_string(static_string_to_char_slice!(r#""hello"world"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_invalid_with_trailing_word()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'hello'world"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_invalid_containing_unescaped_double_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#""some"thing""#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_containing_unescaped_single_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#""some'thing""#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_containing_unescaped_double_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'some"thing'"#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_invalid_containing_unescaped_single_quote()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'some'thing'"#))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_containing_bracket_opening()
            {
                match deserialize_string(static_string_to_char_slice!(r#""hello[world!""#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_containing_bracket_closing()
            {
                match deserialize_string(static_string_to_char_slice!(r#""hello]world!""#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_containing_brace_opening()
            {
                match deserialize_string(static_string_to_char_slice!(r#""hello{world!""#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_containing_brace_closing()
            {
                match deserialize_string(static_string_to_char_slice!(r#""hello}world!""#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_unquoted_multiple_words()
            {
                match deserialize_string(static_string_to_char_slice!("test test"))
                {
                    Ok(_) => panic!("deserialize_string returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_multiple_words()
            {
                match deserialize_string(static_string_to_char_slice!(r#""hello world!""#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value == "hello world!" { return }
                            else { panic!("deserialize_string returned `{}` instead of `hello world!`!", value) }
                        }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_multiple_words()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'hello world!'"#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value == "hello world!" { return }
                            else { panic!("deserialize_string returned `{}` instead of `hello world!`!", value) }
                        }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_unquoted_null()
            {
                match deserialize_string(static_string_to_char_slice!("null"))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Null = obj { return }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_null()
            {
                match deserialize_string(static_string_to_char_slice!(r#""null""#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value == "null" { return }
                            else { panic!("deserialize_string returned `{}` instead of `null`!", value) }
                        }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_null()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'null'"#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value == "null" { return }
                            else { panic!("deserialize_string returned `{}` instead of `null`!", value) }
                        }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_unquoted_one_word()
            {
                match deserialize_string(static_string_to_char_slice!("something"))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value == "something" { return }
                            else { panic!("deserialize_string returned `{}` instead of `something`!", value) }
                        }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_one_word()
            {
                match deserialize_string(static_string_to_char_slice!(r#""something""#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value == "something" { return }
                            else { panic!("deserialize_string returned `{}` instead of `something`!", value) }
                        }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_single_quoted_one_word()
            {
                match deserialize_string(static_string_to_char_slice!(r#"'something'"#))
                {
                    Ok(obj) =>
                    {
                        if let JsonObject::Value { value } = obj
                        {
                            if value == "something" { return }
                            else { panic!("deserialize_string returned `{}` instead of `something`!", value) }
                        }
                        else { panic!("deserialize_string returned an incorrect JsonObject type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_string_double_quoted_containing_escaped_symbols()
            {
                match deserialize_string(static_string_to_char_slice!(r#""\u0009\u000a\u000d\"\\""#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if value == "\t\n\r\"\\" { return }
                            else { panic!("deserialize_string returned `{}`[{}] while `\\t\\n\\r\\\"\\`[5] (escaped) was expected!", value, value.len()) }
                        }
                        else { panic!("deserialize_string returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }
        }

        mod field
        {
            use super::*;

            #[test]
            fn test_deserialize_field_empty_string()
            {
                match deserialize_field(&Vec::new())
                {
                    Ok(_) => panic!("deserialize_field returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_field_only_colon()
            {
                match deserialize_field(static_string_to_char_slice!(":"))
                {
                    Ok(_) => panic!("deserialize_field returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_field_key_double_quoted_missing_value()
            {
                match deserialize_field(static_string_to_char_slice!(r#""key":"#))
                {
                    Ok(_) => panic!("deserialize_field returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_field_key_missing_value_double_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#":"value""#))
                {
                    Ok(_) => panic!("deserialize_field returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_field_key_null_value_unquoted()
            {
                match deserialize_field(static_string_to_char_slice!("null:b"))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "null" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `null:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_unquoted_value_unquoted()
            {
                match deserialize_field(static_string_to_char_slice!("a:b"))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_double_quoted_value_unquoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#""a":b"#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_single_quoted_value_unquoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#"'a':b"#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_unquoted_value_double_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#"a:"b""#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_double_quoted_value_double_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#""a":"b""#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_single_quoted_value_double_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#"'a':"b""#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_unquoted_value_single_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#"a:'b'"#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_double_quoted_value_single_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#""a":'b'"#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_single_quoted_value_single_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#"'a':'b'"#))
                {
                    Ok((key, json)) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if key == "a" && value == "b" { return }
                            else { panic!("deserialize_field returned `{}:{}` instead of `a:b`!", key, value) }
                        }
                        else { panic!("deserialize_field returned field of unexpected type!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_object_value_double_quoted()
            {
                match deserialize_field(static_string_to_char_slice!("{key:null}:null"))
                {
                    Ok(_) => panic!("deserialize_field returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_field_key_array_value_double_quoted()
            {
                match deserialize_field(static_string_to_char_slice!(r#"[1,2]:"null""#))
                {
                    Ok(_) => panic!("deserialize_field returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_field_key_double_quoted_value_array()
            {
                match deserialize_field(static_string_to_char_slice!(r#""key":[1,2,3]"#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_field_key_double_quoted_value_object()
            {
                match deserialize_field(static_string_to_char_slice!(r#""key":{"inner-key":"value"}"#))
                {
                    Ok(_) => return,
                    Err(reason) => panic!("{}", reason),
                }
            }
        }

        mod array
        {
            use super::*;

            #[test]
            fn test_deserialize_array_empty_string()
            {
                match deserialize_array(&Vec::new())
                {
                    Ok(_) => panic!("deserialize_array returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_array_invalid_leading_character()
            {
                match deserialize_array(static_string_to_char_slice!("f[1,2,3]"))
                {
                    Ok(_) => panic!("deserialize_array returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_array_invalid_trailing_character()
            {
                match deserialize_array(static_string_to_char_slice!("[1,2,3]f"))
                {
                    Ok(_) => panic!("deserialize_array returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_array_empty_array()
            {
                match deserialize_array(static_string_to_char_slice!("[]"))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.is_empty() { return }
                            else { panic!("deserialize_array returned non-empty array for input `[]`!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_single_string_without_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"["hello world!"]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 1 { panic!("deserialize_array returned an array of `{}` element(s) while `1` element was expected!", array.len()) }
                            if let JsonObject::Value { value } = array.first().unwrap()
                            {
                                if value == "hello world!" { return }
                                else { panic!(r#"deserialize_array returned `[{}]` instead of `["hello world!"]`!"#, value) }
                            }
                            else { panic!("deserialize_array returned an array containing JsonObject of unexpected variant!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_single_object_without_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"[{"key":null}]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 1 { panic!("deserialize_array returned an array of `{}` element(s) while `1` element was expected!", array.len()) }
                            if let JsonObject::Object { fields } = array.first().unwrap()
                            {
                                if fields.len() == 1 && let JsonObject::Null = fields.get("key").unwrap() { return }
                                else { panic!(r#"deserialize_array didn't return `[{{"key":null}}]`!"#) }
                            }
                            else { panic!("deserialize_array returned an array containing JsonObject of unexpected variant!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_single_array_without_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!("[[null]]"))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 1 { panic!("deserialize_array returned an array of `{}` element(s) while `1` element was expected!", array.len()) }
                            if let JsonObject::Array { array } = array.first().unwrap()
                            {
                                if array.len() == 1 && let JsonObject::Null = array.first().unwrap() { return }
                                else { panic!(r#"deserialize_array didn't return `[[null]]`!"#) }
                            }
                            else { panic!("deserialize_array returned an array containing JsonObject of unexpected variant!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_single_string_with_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"["hello world!",]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 1 { panic!("deserialize_array returned an array of `{}` element(s) while `1` element was expected!", array.len()) }
                            if let JsonObject::Value { value } = array.first().unwrap()
                            {
                                if value == "hello world!" { return }
                                else { panic!(r#"deserialize_array returned `[{}]` instead of `["hello world!"]`!"#, value) }
                            }
                            else { panic!("deserialize_array returned an array containing JsonObject of unexpected variant!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_single_object_with_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"[{"key":null},]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 1 { panic!("deserialize_array returned an array of `{}` element(s) while `1` element was expected!", array.len()) }
                            if let JsonObject::Object { fields } = array.first().unwrap()
                            {
                                if fields.len() == 1 && let JsonObject::Null = fields.get("key").unwrap() { return }
                                else { panic!(r#"deserialize_array didn't return `[{{"key":null}}]`!"#) }
                            }
                            else { panic!("deserialize_array returned an array containing JsonObject of unexpected variant!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_single_array_with_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!("[[null],]"))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 1 { panic!("deserialize_array returned an array of `{}` element(s) while `1` element was expected!", array.len()) }
                            if let JsonObject::Array { array } = array.first().unwrap()
                            {
                                if array.len() == 1 && let JsonObject::Null = array.first().unwrap() { return }
                                else { panic!(r#"deserialize_array didn't return `[[null]]`!"#) }
                            }
                            else { panic!("deserialize_array returned an array containing JsonObject of unexpected variant!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_string_object_array_without_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"["null",{"key":null},[]]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 3 { panic!("deserialize_array returned an array of length != 3!") }
                            else if let JsonObject::Value { value: string } = &array[0] && let JsonObject::Object { fields: object } = &array[1] && let JsonObject::Array { array: array_inner } = &array[2]
                            {
                                if string == "null" && let Some(object_value) = object.get("key") && let JsonObject::Null = object_value && array_inner.is_empty() { return }
                                else { panic!("deserialize_array returned an array containing 3 elements with unexpected values!") }
                            }
                            else { panic!("deserialize_array returned an array containing unexpected JsonObject variant(s)!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_object_string_array_without_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"[{"key":null},"null",[]]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 3 { panic!("deserialize_array returned an array of length != 3!") }
                            else if let JsonObject::Object { fields: object } = &array[0] && let JsonObject::Value { value: string } = &array[1] && let JsonObject::Array { array: array_inner } = &array[2]
                            {
                                if string == "null" && let Some(object_value) = object.get("key") && let JsonObject::Null = object_value && array_inner.is_empty() { return }
                                else { panic!("deserialize_array returned an array containing 3 elements with unexpected values!") }
                            }
                            else { panic!("deserialize_array returned an array containing unexpected JsonObject variant(s)!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_array_object_string_without_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"[[],{"key":null},"null"]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 3 { panic!("deserialize_array returned an array of length != 3!") }
                            else if let JsonObject::Array { array: array_inner } = &array[0] &&  let JsonObject::Object { fields: object } = &array[1] && let JsonObject::Value { value: string } = &array[2]
                            {
                                if string == "null" && let Some(object_value) = object.get("key") && let JsonObject::Null = object_value && array_inner.is_empty() { return }
                                else { panic!("deserialize_array returned an array containing 3 elements with unexpected values!") }
                            }
                            else { panic!("deserialize_array returned an array containing unexpected JsonObject variant(s)!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_string_object_array_with_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"["null",{"key":null},[],]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 3 { panic!("deserialize_array returned an array of length != 3!") }
                            else if let JsonObject::Value { value: string } = &array[0] && let JsonObject::Object { fields: object } = &array[1] && let JsonObject::Array { array: array_inner } = &array[2]
                            {
                                if string == "null" && let Some(object_value) = object.get("key") && let JsonObject::Null = object_value && array_inner.is_empty() { return }
                                else { panic!("deserialize_array returned an array containing 3 elements with unexpected values!") }
                            }
                            else { panic!("deserialize_array returned an array containing unexpected JsonObject variant(s)!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_object_string_array_with_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"[{"key":null},"null",[],]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 3 { panic!("deserialize_array returned an array of length != 3!") }
                            else if let JsonObject::Object { fields: object } = &array[0] && let JsonObject::Value { value: string } = &array[1] && let JsonObject::Array { array: array_inner } = &array[2]
                            {
                                if string == "null" && let Some(object_value) = object.get("key") && let JsonObject::Null = object_value && array_inner.is_empty() { return }
                                else { panic!("deserialize_array returned an array containing 3 elements with unexpected values!") }
                            }
                            else { panic!("deserialize_array returned an array containing unexpected JsonObject variant(s)!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_array_containing_array_object_string_with_trailing_comma()
            {
                match deserialize_array(static_string_to_char_slice!(r#"[[],{"key":null},"null",]"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 3 { panic!("deserialize_array returned an array of length != 3!") }
                            else if let JsonObject::Array { array: array_inner } = &array[0] &&  let JsonObject::Object { fields: object } = &array[1] && let JsonObject::Value { value: string } = &array[2]
                            {
                                if string == "null" && let Some(object_value) = object.get("key") && let JsonObject::Null = object_value && array_inner.is_empty() { return }
                                else { panic!("deserialize_array returned an array containing 3 elements with unexpected values!") }
                            }
                            else { panic!("deserialize_array returned an array containing unexpected JsonObject variant(s)!") }
                        }
                        else { panic!("deserialize_array returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }
        }

        mod object
        {
            use super::*;

            #[test]
            fn test_deserialize_object_empty_string()
            {
                match deserialize_object(&Vec::new())
                {
                    Ok(_) => panic!("deserialize_object returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_object_invalid_leading_character()
            {
                match deserialize_object(static_string_to_char_slice!("!]"))
                {
                    Ok(_) => panic!("deserialize_object returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_object_invalid_trailing_character()
            {
                match deserialize_object(static_string_to_char_slice!("[!"))
                {
                    Ok(_) => panic!("deserialize_object returned Ok!"),
                    Err(_) => return,
                }
            }

            #[test]
            fn test_deserialize_object_empty_object()
            {
                match deserialize_object(static_string_to_char_slice!("{}"))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.is_empty() { return }
                            else { panic!("deserialize_object returned a non-empty object!") }
                        }
                        else { panic!("deserialize_object returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_object_single_field_without_trailing_comma()
            {
                match deserialize_object(static_string_to_char_slice!(r#"{"key":null}"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.len() == 1
                            {
                                if let Some(value) = fields.get("key") && let JsonObject::Null = value { return }
                                else { panic!("deserialize_object returned JsonObject with unexpected contents!") }
                            }
                            else { panic!("deserialize_object returned JsonObject with an unexpected number of fields!") }
                        }
                        else { panic!("deserialize_object returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_object_single_field_with_trailing_comma()
            {
                match deserialize_object(static_string_to_char_slice!(r#"{"key":null,}"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.len() == 1
                            {
                                if let Some(value) = fields.get("key") && let JsonObject::Null = value { return }
                                else { panic!("deserialize_object returned JsonObject with unexpected contents!") }
                            }
                            else { panic!("deserialize_object returned JsonObject with an unexpected number of fields!") }
                        }
                        else { panic!("deserialize_object returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_object_double_field_without_trailing_comma()
            {
                match deserialize_object(static_string_to_char_slice!(r#"{"first":1,"second":2}"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.len() == 2
                            {
                                if let Some(value_first) = fields.get("first") && let JsonObject::Value { value: value_first_value } = value_first && value_first_value == "1"
                                    && let Some(value_second) = fields.get("second") && let JsonObject::Value  { value: value_second_value } = value_second && value_second_value == "2"
                                {
                                    return
                                }
                                else { panic!("deserialize_object returned JsonObject with unexpected contents!") }
                            }
                            else { panic!("deserialize_object returned JsonObject with an unexpected number of fields!") }
                        }
                        else { panic!("deserialize_object returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_object_double_field_with_trailing_comma()
            {
                match deserialize_object(static_string_to_char_slice!(r#"{"first":1,"second":2,}"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.len() == 2
                            {
                                if let Some(value_first) = fields.get("first") && let JsonObject::Value { value: value_first_value } = value_first && value_first_value == "1"
                                    && let Some(value_second) = fields.get("second") && let JsonObject::Value { value: value_second_value } = value_second && value_second_value == "2"
                                {
                                    return
                                }
                                else { panic!("deserialize_object returned JsonObject with unexpected contents!") }
                            }
                            else { panic!("deserialize_object returned JsonObject with an unexpected number of fields!") }
                        }
                        else { panic!("deserialize_object returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_object_triple_field_without_trailing_comma()
            {
                match deserialize_object(static_string_to_char_slice!(r#"{"first":1,"second":2,"third":3}"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.len() == 3
                            {
                                if let Some(value_first) = fields.get("first") && let JsonObject::Value { value: value_first_value } = value_first && value_first_value == "1"
                                    && let Some(value_second) = fields.get("second") && let JsonObject::Value  { value: value_second_value } = value_second && value_second_value == "2"
                                    && let Some(value_third) = fields.get("third") && let JsonObject::Value  { value: value_third_value } = value_third && value_third_value == "3"
                                {
                                    return
                                }
                                else { panic!("deserialize_object returned JsonObject with unexpected contents!") }
                            }
                            else { panic!("deserialize_object returned JsonObject with an unexpected number of fields!") }
                        }
                        else { panic!("deserialize_object returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_deserialize_object_triple_field_with_trailing_comma()
            {
                match deserialize_object(static_string_to_char_slice!(r#"{"first":1,"second":2,"third":3,}"#))
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.len() == 3
                            {
                                if let Some(value_first) = fields.get("first") && let JsonObject::Value { value: value_first_value } = value_first && value_first_value == "1"
                                    && let Some(value_second) = fields.get("second") && let JsonObject::Value { value: value_second_value } = value_second && value_second_value == "2"
                                    && let Some(value_third) = fields.get("third") && let JsonObject::Value { value: value_third_value } = value_third && value_third_value == "3"
                                {
                                    return
                                }
                                else { panic!("deserialize_object returned JsonObject with unexpected contents!") }
                            }
                            else { panic!("deserialize_object returned JsonObject with an unexpected number of fields!") }
                        }
                        else { panic!("deserialize_object returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }
        }

        mod json_object_try_from_string_slice
        {
            use super::*;

            macro_rules! string_slice_to_json_object
            {
                ($input:literal) =>
                {
                    JsonObject::try_from($input)
                };
            }

            #[test]
            fn test_json_object_empty_string()
            {
                match string_slice_to_json_object!("")
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Null = json { return }
                        else { panic!("JsonObject::try_from(…) returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_json_object_one_character_string()
            {
                match string_slice_to_json_object!("1")
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if value == "1" { return }
                            else { panic!("JsonObject::try_from(…) returned `{}` instead of `1`!", value) }
                        }
                        else { panic!("JsonObject::try_from(…) returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_json_object_object_with_single_field()
            {
                match string_slice_to_json_object!(r#"{"key":null}"#)
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Object { fields } = json
                        {
                            if fields.len() != 1 { panic!("JsonObject::try_from(…) returned JsonObject with an unexpected number of fields!") }
                            else if let JsonObject::Null = fields.get("key").unwrap() { return }
                            else { panic!("JsonObject::try_from(…) returned JsonObject with an unexpected field!") }
                        }
                        else { panic!("JsonObject::try_from(…) returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_json_object_array_with_single_item()
            {
                match string_slice_to_json_object!("[null]")
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Array { array } = json
                        {
                            if array.len() != 1 { panic!("JsonObject::try_from(…) returned an array of an unexpected length!") }
                            else if let JsonObject::Null = array.first().unwrap() { return }
                            else { panic!("JsonObject::try_from(…) returned an array containing an unexpected item!") }
                        }
                        else { panic!("JsonObject::try_from(…) returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }

            #[test]
            fn test_json_object_string()
            {
                match string_slice_to_json_object!(r#""hello world!""#)
                {
                    Ok(json) =>
                    {
                        if let JsonObject::Value { value } = json
                        {
                            if value == "hello world!" { return }
                            else { panic!("JsonObject::try_from(…) returned `{}` instead of `hello world!`!", value) }
                        }
                        else { panic!("JsonObject::try_from(…) returned JsonObject of an unexpected variant!") }
                    },
                    Err(reason) => panic!("{}", reason),
                }
            }
        }
    }

    mod serializer
    {
        use super::*;

        mod json_object_to_json
        {
            use super::*;

            #[test]
            fn test_serialize_null()
            {
                assert_eq!(JsonObject::Null.to_json(), "null")
            }

            #[test]
            fn test_serialize_value_string()
            {
                assert_eq!(JsonObject::Value { value: "hello world!".to_string() }.to_json(), r#""hello world!""#)
            }

            #[test]
            fn test_serialize_value_u128_positive()
            {
                assert_eq!(JsonObject::Value { value: 8u128.to_string() }.to_json(), "8")
            }

            #[test]
            fn test_serialize_value_i128_negative()
            {
                assert_eq!(JsonObject::Value { value: (-8i128).to_string() }.to_json(), "-8")
            }

            #[test]
            fn test_serialize_value_f64_negative()
            {
                assert_eq!(JsonObject::Value { value: (-8.8f64).to_string() }.to_json(), "-8.8")
            }

            #[test]
            fn test_serialize_object_empty()
            {
                assert_eq!(JsonObject::Object { fields: BTreeMap::new() }.to_json(), r#"{}"#)
            }

            #[test]
            fn test_serialize_object_with_one_field()
            {
                let mut fields = BTreeMap::new();
                fields.insert("key".to_string(), JsonObject::Null);
                assert_eq!(JsonObject::Object { fields }.to_json(), r#"{"key":null}"#)
            }

            #[test]
            fn test_serialize_object_with_three_fields()
            {
                let mut fields = BTreeMap::new();
                fields.insert("first".to_string(), JsonObject::Value { value: "1".to_string() });
                fields.insert("second".to_string(), JsonObject::Value { value: "2".to_string() });
                fields.insert("third".to_string(), JsonObject::Value { value: "3".to_string() });
                assert_eq!(JsonObject::Object { fields }.to_json(), r#"{"first":1,"second":2,"third":3}"#)
            }

            #[test]
            fn test_serialize_array_empty()
            {
                assert_eq!(JsonObject::Array { array: Vec::new() }.to_json(), r#"[]"#)
            }

            #[test]
            fn test_serialize_array_with_one_field()
            {
                assert_eq!(JsonObject::Array { array: vec![ JsonObject::Null ] }.to_json(), r#"[null]"#)
            }

            #[test]
            fn test_serialize_array_with_three_fields()
            {
                assert_eq!
                (
                    JsonObject::Array { array: vec![ JsonObject::Value { value: "1".to_string() }, JsonObject::Value { value: "2".to_string() }, JsonObject::Value { value: "3".to_string() } ] }.to_json(),
                    r#"[1,2,3]"#
                )
            }
        }

        mod string
        {
            use super::*;

            #[test]
            fn test_serialize_string_empty()
            {
                assert_eq!(serialize_string(""), "\"\"")
            }

            #[test]
            fn test_serialize_string_without_escaped_characters()
            {
                assert_eq!(serialize_string("hello world!"), "\"hello world!\"")
            }

            #[test]
            fn test_serialize_string_with_escaped_characters()
            {
                assert_eq!(serialize_string("\t\n\r\"\\"), r#""\u0009\u000a\u000d\"\\""#)
            }
        }

        mod trimmable
        {
            use super::*;

            #[test]
            fn test_trimmable()
            {
                assert_eq!("\u{000B}\u{000C}\u{0085}hello      world!\u{2028}\u{2029}\t\n\t\r ".chars().collect::<Vec<char>>().as_slice().trim().iter().collect::<String>(), "hello      world!")
            }
        }
    }
}
