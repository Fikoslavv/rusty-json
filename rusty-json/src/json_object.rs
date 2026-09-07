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
    // println!("Deserializing string => `{}`", value.iter().collect::<String>());

    let mut string_closing_character: Option<char> = None;
    let mut escaping = false;
    let mut is_string = false;

    for (index, char) in value.iter().enumerate()
    {
        if *char == '\\'
        {
            escaping = !escaping;
        }
        else if *char == ' '
        {
            if !is_string { return Err("Given string is not valid json !"); }
        }
        else if *char == '\'' || *char == '"'
        {
            if index > 0 && index < value.len() - 1 && !escaping { return Err("Given string is not valid json !"); }
            else if string_closing_character.is_none()
            {
                string_closing_character = Some(*char);
                is_string = true;
            }
            else if escaping || *char != string_closing_character.unwrap() { escaping = false; }
            else
            {
                string_closing_character = None;
                if index < value.len() - 1 { return Err("Given string is not valid json !"); }
            }
        }
    }

    if is_string { Ok(JsonObject::Value { value: value[1..value.len() - 1].iter().collect::<String>() }) }
    else if value.len() == 4 && value.iter().zip([ 'n', 'u', 'l', 'l' ]).all(|(l, r)| *l == r) { Ok(JsonObject::Null) }
    else { Ok(JsonObject::Value { value: value.iter().collect::<String>() }) }
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
