use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Datum {
    Null,
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScalarType {
    ///
    /// MySQL Numeric Data Types: https://dev.mysql.com/doc/refman/8.0/en/numeric-types.html
    ///
    // Integer Types: https://dev.mysql.com/doc/refman/8.0/en/integer-types.html
    TinyInt,
    SmallInt,
    MediumInt,
    Int,
    BigInt,
    // Fixed-Point Types: https://dev.mysql.com/doc/refman/8.0/en/fixed-point-types.html
    // Decimal is used for storing exact decimal values with a fixed precision and scale.
    Decimal,
    // Floating-Point Types: https://dev.mysql.com/doc/refman/8.0/en/floating-point-types.html
    // Float and Double is used for storing approximate floating-point values.
    Float,
    Double,
    // Bit-Value Type: https://dev.mysql.com/doc/refman/8.0/en/bit-type.html
    // Bit is used for storing bit values.
    Bit,
    ///
    /// MySQL String Data Types: https://dev.mysql.com/doc/refman/8.0/en/string-types.html
    ///
    // Char and Varchar: https://dev.mysql.com/doc/refman/8.0/en/char.html
    Char,
    VarChar,
    // Binary and VarBinary: https://dev.mysql.com/doc/refman/8.0/en/binary-varbinary.html
    Binary,
    VarBinary,
    // Blob and Text: https://dev.mysql.com/doc/refman/8.0/en/blob.html
    TinyBlob,
    MediumBlob,
    LongBlob,
    TinyText,
    MediumText,
    Text,
    LongText,
    // Enum type: https://dev.mysql.com/doc/refman/8.0/en/enum.html
    Enum(Vec<String>),
    // Set type: https://dev.mysql.com/doc/refman/8.0/en/set.html
    Set(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnDesc {
    pub scalar_type: ScalarType,
    pub nullable: bool,
    pub name: String,
}

impl ColumnDesc {
    pub fn new(
        name: String,
        data_type: String,
        nullable: String,
    ) -> Result<Self> {
        let nullable = match nullable.as_str() {
            "YES" => true,
            "NO" => false,
            _ => return Err(anyhow!("Invalid nullable value: {}", nullable)),
        };
        let scalar_type = match data_type.as_str() {
            "tinyint" => ScalarType::TinyInt,
            "smallint" => ScalarType::SmallInt,
            "mediumint" => ScalarType::MediumInt,
            "int" => ScalarType::Int,
            "bigint" => ScalarType::BigInt,
            "float" => ScalarType::Float,
            "double" => ScalarType::Double,
            "bit" => ScalarType::Bit,
            "char" => ScalarType::Char,
            "varchar" => ScalarType::VarChar,
            "binary" => ScalarType::Binary,
            "varbinary" => ScalarType::VarBinary,
            "tinyblob" => ScalarType::TinyBlob,
            "mediumblob" => ScalarType::MediumBlob,
            "longblob" => ScalarType::LongBlob,
            "tinytext" => ScalarType::TinyText,
            "mediumtext" => ScalarType::MediumText,
            "text" => ScalarType::Text,
            "longtext" => ScalarType::LongText,
            _ => return Err(anyhow!("Invalid scalar type: {}", data_type)),
        };
        Ok(ColumnDesc {
            name,
            scalar_type,
            nullable,
        })
    }
}

#[derive(Clone, Debug)]
pub struct TableDesc {
    pub columns: Vec<ColumnDesc>,
}

impl PartialEq for TableDesc {
    fn eq(&self, other: &Self) -> bool {
        (self.columns.len() == other.columns.len())
            && self
                .columns
                .iter()
                .zip(other.columns.iter())
                .all(|(a, b)| a == b)
    }
}
