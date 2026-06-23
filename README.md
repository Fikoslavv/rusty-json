# rusty-json

*rusty-json* is a proof-of-concept json serializer and deserializer library. It is not intended to be used in production for the time being.

# How do I use it anyway?

## Install

You clone the project, switch to branch you want to use: _main_ or _dev-macros-derive_ (for procedural macros support).
Then you add this line to your `Cargo.toml` file's dependecies section ↓

```toml
rusty-json = { git = "https://github.com/Fikoslavv/rusty-json" }
```

or this line for procedural macros support ↓

```toml
rusty-json = { git = "https://github.com/Fikoslavv/rusty-json", branch = "dev-macros-derive", features = [ "derive" ] }
```

## Usage

You can choose to use procedural macros to make a struct serializable/deserializable or implement `TryFrom<JsonObject>` and `Into<JsonObject>` for the type (and preferably for `&type` for easier usage).

### An example of manual implementation of `TryFrom<JsonObject>` and `Into<JsonObject>`

```rust

struct MyStruct
{
    string: String,
    float: f64,
}

impl Into<JsonObject> for MyStruct
{
    fn into(self) -> JsonObject
    {
        let mut fields: BTreeMap<String, JsonObject> = BTreeMap::new();
        fields.insert("string".to_string(), JsonObject::Value { value: self.string });
        fields.insert("float".to_string(), JsonObject::Value { value: self.float.to_string() });

        JsonObject::Object { fields }
    }
}

impl TryFrom<JsonObject> for MyStruct
{
    type Error = String;

    fn try_from(json: JsonObject) -> Result<Self, Self::Error>
    {
        let fields = match json
        {
            JsonObject::Object { fields } => fields,
            _ => return Err("MyStruct may only be deserialized from an object!".to_string()),
        };

        let obj_string = if let Some(value) = fields.get("string") { value } else { return Err("MyStruct requires field `string` to be deserialized!".to_string()) };
        let obj_float = if let Some(value) = fields.get("float") { value } else { return Err("MyStruct requires field `float` to be deserialized!".to_string()) };

        let string = match obj_string
        {
            JsonObject::Value { value } => value,
            _ => return Err("MyStruct requires entry `string` to be a field!".to_string()),
        };

        let float = match obj_float
        {
            JsonObject::Value { value } => value,
            _ => return Err("MyStruct requires entry `float` to be a field!".to_string()),
        };

        let float = match float.parse::<f64>()
        {
            Ok(value) => value,
            Err(_) => return Err("MyStruct requires field `float` to be a float!".to_string()),
        };

        Ok(Self { string: string.to_owned(), float })
    }
}

```

### An example of procedural macros usage

```rust

#[derive(JsonSerialize, JsonDeserialize)]
struct MyStruct
{
    string: String,
    float: f64,
}

```

### Serialization and deserialization

The following functions show how to serialize and deserialize an object to and from JSON.

```rust

fn demo_serialization<T>(sth: T) where T : Into<JsonObject>, for <'a> &'a T : Into<JsonObject>
{
    let _: String = to_json(&sth);
    let _: String = sth.into().into_json();
}

fn demo_deserialization<T>(str: String) where T : TryFrom<JsonObject, Error = String>, for <'a> &'a T : TryFrom<JsonObject>, <T as TryFrom<rusty_json::JsonObject>>::Error: std::fmt::Debug
{
    let _: T = from_json(&str).expect("Failed to parse deserialize json!");
    let _: T = JsonObject::try_from(&str[..]).expect("Failed to deserialize json!").try_into().expect("Failed to parse JsonObject to T!");
}

```
