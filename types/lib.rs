use serde::ser::{Serialize, SerializeMap, Serializer};

pub struct XDatum(mysql_async::Value);

impl From<serde_json::Value> for XDatum {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::String(s) => {
                return XDatum(mysql_async::Value::Bytes(s.into_bytes()));
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    return XDatum(mysql_async::Value::Int(i));
                }
                if let Some(u) = n.as_u64() {
                    return XDatum(mysql_async::Value::UInt(u));
                }
                if let Some(f) = n.as_f64() {
                    return XDatum(mysql_async::Value::Double(f));
                }
                unimplemented!("Number type not supported");
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

impl From<XDatum> for mysql_async::Value {
    fn from(v: XDatum) -> Self {
        v.0
    }
}

impl From<&mysql_async::Value> for XDatum {
    fn from(v: &mysql_async::Value) -> Self {
        XDatum(v.clone())
    }
}

pub struct XValue {
    pub datum: XDatum,
    pub ty: mysql_async::consts::ColumnType,
}

impl Serialize for XValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.datum.0 {
            mysql_async::Value::Bytes(bytes) if self.ty.is_character_type() => {
                serializer
                    .serialize_str(&String::from_utf8(bytes.clone()).unwrap())
            }
            mysql_async::Value::Int(i) => serializer.serialize_i64(*i),
            mysql_async::Value::UInt(i) => serializer.serialize_u64(*i),
            mysql_async::Value::Float(f) => serializer.serialize_f32(*f),
            mysql_async::Value::Double(f) => serializer.serialize_f64(*f),
            _ => {
                unimplemented!()
            }
        }
    }
}

pub struct XColumn {
    pub name: String,
    pub value: XValue,
}

pub struct XRow(pub Vec<XColumn>);

impl Serialize for XRow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for column in &self.0 {
            map.serialize_entry(&column.name, &column.value)?;
        }
        map.end()
    }
}
