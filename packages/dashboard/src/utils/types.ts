export type PrimitiveType = number | string | boolean | null;

export type MySQLFieldType =
  // numeric data types
  | "tinyint"
  // | "smallint"
  // | "mediumint"
  // | "int"
  | "bigint"
  // | "decimal"
  // | "numeric"
  // | "float"
  | "double"
  // | "bit"
  // date and time data types
  // | "date"
  // | "time"
  | "datetime"
  // | "timestamp"
  // | "year"
  // string data types
  // | "char"
  | "varchar"
  // | "binary"
  // | "varbinary"
  // | "blob"
  | "text";
// | "enum"
// | "set"
// json
// | "json";

export function mysqlToFieldType(t: MySQLFieldType, extra: string): FieldType {
  switch (t) {
    case "bigint":
      if (extra.toUpperCase() === "AUTO_INCREMENT") {
        return "int64Identity";
      } else {
        return "int64";
      }
    case "tinyint":
      return "bool";
    case "double":
      return "float64";
    case "datetime":
      return "datetime";
    case "varchar":
      return "varchar(255)";
    case "text":
      return "text";
  }
}

export type FieldType = UserFieldType | "int64Identity" | "NotDefined";

export type UserFieldType =
  | "int64"
  | "float64"
  | "bool"
  | "datetime"
  | "varchar(255)"
  | "text";

export const USER_FIELD_TYPES: UserFieldType[] = [
  "int64",
  "float64",
  "bool",
  "datetime",
  "varchar(255)",
  "text",
];

export function displayFieldType(t: FieldType) {
  switch (t) {
    case "int64Identity":
      return "int64 Identity";
    default:
      return t;
  }
}

export function displayToFieldType(t: string): FieldType {
  switch (t) {
    case "int64":
      return "int64";
    case "int64 Identity":
      return "int64Identity";
    case "float64":
      return "float64";
    case "bool":
      return "bool";
    case "datetime":
      return "datetime";
    case "varchar(255)":
      return "varchar(255)";
    case "text":
      return "text";
    default:
      throw new Error(`Invalid type: ${t}`);
  }
}

// mysql 8.0.13 can have default value for TEXT when
// it is defined as an expression.
export type DefaultValue =
  | { typ: UserFieldType; value: string }
  | { typ: "expr"; value: string }
  | { typ: "NULL"; value: "" }
  | { typ: "NotDefined"; value: "" };

export function displayDefaultValue(defaultValue: DefaultValue) {
  switch (defaultValue.typ) {
    case "NotDefined":
      return "";
    case "NULL":
      return "NULL";
    default:
      return defaultValue.value;
  }
}

// http response to dx default value
export function primitiveToDefaultValue(
  v: PrimitiveType,
  fieldType: FieldType,
  isNullable: boolean
): DefaultValue {
  // if the column is nullable, then a default value of "null" means it is "NULL".
  // if the column is not-nullable, then a default value of "null" means the default value is empty.
  if (v === null) {
    if (isNullable) {
      return { typ: "NULL", value: "" };
    } else {
      return { typ: "NotDefined", value: "" };
    }
  }

  switch (fieldType) {
    case "int64":
      return { typ: "int64", value: (v as number).toString() };
    case "int64Identity":
      return { typ: "NotDefined", value: "" };
    case "float64":
      return { typ: "float64", value: (v as number).toString() };
    case "bool":
      return { typ: "bool", value: (v as boolean).toString() };
    case "datetime":
      const value = v as string;
      if (value.includes("CURRENT_TIMESTAMP") || value.includes("NOW")) {
        return { typ: "expr", value: value };
      } else {
        return { typ: "datetime", value: v as string };
      }
    case "varchar(255)":
      return { typ: "varchar(255)", value: v as string };
    case "text":
      return { typ: "expr", value: v as string };
    case "NotDefined":
      throw new Error("field not defined, cannot have a default value");
  }
}

// collect input value from the form
export function stringToDefaultValue(
  fieldType: FieldType,
  v: string
): DefaultValue {
  if (v === "NULL") {
    return { typ: "NULL", value: "" };
  }

  if (v === "") {
    return { typ: "NotDefined", value: "" };
  }

  switch (fieldType) {
    case "int64":
      return { typ: "int64", value: v };
    case "int64Identity":
      throw new Error("int64_identity cannot have a default value");
    case "float64":
      return { typ: "float64", value: v };
    case "bool":
      return { typ: "bool", value: v };
    case "datetime":
      return { typ: "datetime", value: v };
    case "varchar(255)":
      return { typ: "varchar(255)", value: v };
    case "text":
      return { typ: "expr", value: `(${v})` };
    case "NotDefined":
      throw new Error("field not defined, cannot have a default value");
  }
}
