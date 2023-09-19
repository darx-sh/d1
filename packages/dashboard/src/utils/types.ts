export type PrimitiveType = PrimitiveNotNull | null;
export type PrimitiveNotNull = number | string | boolean;

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
        return "int64 Auto Increment";
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

export type FieldType =
  | "int64"
  | "float64"
  | "bool"
  | "datetime"
  | "varchar(255)"
  | "text"
  | "int64 Auto Increment"
  | "Not Defined";

export const FieldTypeList: FieldType[] = [
  "int64",
  "float64",
  "bool",
  "datetime",
  "varchar(255)",
  "text",
  "int64 Auto Increment",
  "Not Defined",
];

export type DatumNotNull =
  | { typ: "int64"; value: number }
  | { typ: "float64"; value: number }
  | { typ: "bool"; value: boolean }
  | { typ: "datetime"; value: string }
  | { typ: "varchar(255)"; value: string }
  | { typ: "text"; value: string }
  | { typ: "int64AutoIncrement"; value: number };

export type Datum = DatumNotNull | { typ: "NULL" };

export type DefaultValue =
  | { typ: "int64"; value: number }
  | { typ: "float64"; value: number }
  | { typ: "bool"; value: boolean }
  | { typ: "datetime"; value: string }
  | { typ: "varchar(255)"; value: string }
  | { typ: "text"; value: string }
  | { typ: "NULL" }
  | { typ: "expr"; value: string }
  | { typ: "Not Defined" };

// export function defaultValueToDatum(defaultValue: DefaultValue): Datum {
//   if (defaultValue.typ === "Not Defined") {
//     throw new Error("Default value is not defined");
//   } else {
//     return defaultValue;
//   }
// }

export function displayDefaultValue(v: DefaultValue): string {
  switch (v.typ) {
    case "NULL":
      return "NULL";
    case "Not Defined":
      return "Not Defined";
    default:
      return v.value.toString();
  }
}

export function displayDatum(datum: Datum) {
  switch (datum.typ) {
    case "NULL":
      return "NULl";
    case "int64":
      return datum.value.toString();
    case "float64":
      return datum.value.toString();
    case "bool":
      return datum.value.toString();
    case "datetime":
      return datum.value;
    case "varchar(255)":
      return datum.value;
    case "text":
      return datum.value;
    case "int64AutoIncrement":
      return datum.value.toString();
  }
}

export function primitiveToDatum(
  v: PrimitiveType,
  fieldType: FieldType
): Datum {
  if (v === null) {
    return { typ: "NULL" };
  }
  switch (fieldType) {
    case "Not Defined":
      throw new Error("internal error: field type is not defined");
    case "int64 Auto Increment":
    case "int64":
      return { typ: "int64", value: v as number };
    case "float64":
      return { typ: "float64", value: v as number };
    case "bool":
      return { typ: "bool", value: v as boolean };
    case "datetime":
      const value = v as string;
      return { typ: "datetime", value };
    case "varchar(255)":
      return { typ: "varchar(255)", value: v as string };
    case "text":
      return { typ: "text", value: v as string };
  }
}

export function datumToPrimitive(datum: Datum): PrimitiveType {
  switch (datum.typ) {
    case "NULL":
      return null;
    default:
      return datum.value;
  }
}

// http response to dx default value
export function primitiveToDefaultValue(
  v: PrimitiveType,
  fieldType: FieldType
): DefaultValue {
  // if the column is nullable, then a default value of "null" means it is "NULL".
  // if the column is not-nullable, then a default value of "null" means the default value is empty.
  if (v === null) {
    return { typ: "NULL" };
  }

  switch (fieldType) {
    case "Not Defined":
      throw new Error("internal error: field type is not defined");
    case "int64 Auto Increment":
      return { typ: "Not Defined" };
    case "int64":
      return { typ: "int64", value: v as number };
    case "float64":
      return { typ: "float64", value: v as number };
    case "bool":
      return { typ: "bool", value: v as boolean };
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
  }
}

// collect input value from the form
export function stringToDefaultValue(
  fieldType: FieldType,
  v: string
): DefaultValue {
  switch (fieldType) {
    case "Not Defined":
      throw new Error("internal error: field type is not defined");
    case "int64":
    case "int64 Auto Increment":
      return { typ: "int64", value: parseInt(v) };
    case "float64":
      return { typ: "float64", value: parseFloat(v) };
    case "bool":
      return { typ: "bool", value: Boolean(v) };
    case "datetime":
      return { typ: "datetime", value: v };
    case "varchar(255)":
      return { typ: "varchar(255)", value: v };
    case "text":
      return { typ: "text", value: v };
  }
}
