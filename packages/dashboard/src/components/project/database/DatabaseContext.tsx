import { createContext, Dispatch, ReactNode, useContext } from "react";
import { enableMapSet } from "immer";
import { useImmerReducer } from "use-immer";
import { DxDefaultJsonType, PrimitiveTypes } from "~/utils";

enableMapSet();

type DatabaseState = {
  schema: SchemaDef;
  curWorkingTable: { tableName: string; rows: Row[] } | null;
  // table editor's state
  draftTable: TableDef;
  draftTableError: TableDefError;
  draftColumnMarks: ColumnMarkMap;
  editorMod: "Create" | "Update" | "None";
};

export interface Row {
  [key: string]: any[];
}

export interface SchemaDef {
  // key is table name, value is column names
  // column name is ordered by ORDINAL_POSITION
  [key: string]: TableDef;
}

export interface TableDef {
  name: string | null;
  columns: DxColumnType[];
}

export interface TableDefError {
  nameError: string | null;
  columnsError: ColumnError[];
}

export interface ColumnError {
  nameError: string | null;
  fieldTypeError: string | null;
}

export type DxFieldType =
  | "int64"
  | "int64_identity"
  | "float64"
  | "bool"
  | "datetime"
  | "varchar(255)"
  | "text";

export const SELECTABLE_DX_FIELD_TYPES: DxFieldType[] = [
  "int64",
  "float64",
  "bool",
  "datetime",
  "varchar(255)",
  "text",
];

export function displayDxFieldType(t: DxFieldType) {
  switch (t) {
    case "int64_identity":
      return "int64 Identity";
    default:
      return t;
  }
}

export function toDxFieldType(t: string): DxFieldType {
  switch (t) {
    case "int64":
      return "int64";
    case "int64 Identity":
      return "int64_identity";
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

export type DxDefaultValue =
  | { type: "int64"; value: number }
  | { type: "float64"; value: number }
  | { type: "bool"; value: boolean }
  | { type: "datetime"; value: string }
  | { type: "text"; value: string }
  | { type: "expr"; value: string }
  | { type: "NULL" };

export function displayDxDefaultValue(
  fieldType: DxFieldType | null,
  d: DxDefaultValue | null
) {
  // AUTO_INCREMENT type has a NULL default value, but
  // it cannot have a default value.
  if (fieldType === "int64_identity") {
    return "";
  }

  if (d === null) {
    return "NULL";
  }

  // todo: clarify this
  if (d.type === "NULL") {
    return "NULL";
  }

  return d.value.toString();
}

export function defaultValueToJSON(d: DxDefaultValue): DxDefaultJsonType {
  switch (d.type) {
    case "NULL":
      return "NULL";
    case "bool":
      return { bool: d.value };
    case "int64":
      return { int64: d.value };
    case "float64":
      return { float64: d.value };
    case "text":
      return { text: d.value };
    case "datetime":
      return { datetime: d.value };
    case "expr":
      return { expr: d.value };
  }
}

export function toDxDefaultValue(
  v: PrimitiveTypes,
  fieldType: DxFieldType
): DxDefaultValue {
  if (v === null) {
    return { type: "NULL" };
  } else if (fieldType === "int64") {
    return { type: "int64", value: v as number };
  } else if (fieldType === "float64") {
    return { type: "float64", value: v as number };
  } else if (fieldType === "bool") {
    return { type: "bool", value: v as boolean };
  } else if (fieldType === "datetime") {
    const value = v as string;
    if (value.includes("CURRENT_TIMESTAMP") || value.includes("NOW")) {
      return { type: "expr", value: value };
    }
    return { type: "datetime", value: v as string };
  } else if (fieldType === "text") {
    return { type: "text", value: v as string };
  } else {
    throw new Error(`Invalid fieldType`);
  }
}

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

export function mysqlToDxFieldType(
  t: MySQLFieldType,
  extra: string
): DxFieldType {
  switch (t) {
    case "bigint":
      if (extra.toUpperCase() === "AUTO_INCREMENT") {
        return "int64_identity";
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

export interface DxColumnType {
  name: string | null;
  fieldType: DxFieldType | null;
  defaultValue: DxDefaultValue | null;
  isNullable: boolean;
  extra: ExtraColumnOptions | null;
}

type ExtraColumnOptions = "AUTO_INCREMENT" | "ON UPDATE CURRENT_TIMESTAMP(3)";

const DefaultDxColumns: DxColumnType[] = [
  {
    name: "id",
    fieldType: "int64_identity",
    // AUTO_INCREMENT cannot have a default 0
    defaultValue: null,
    isNullable: false,
    extra: "AUTO_INCREMENT",
  },
  {
    name: "created_at",
    fieldType: "datetime",
    defaultValue: { type: "expr", value: "CURRENT_TIMESTAMP(3)" },
    isNullable: false,
    extra: null,
  },
  {
    name: "updated_at",
    fieldType: "datetime",
    defaultValue: { type: "expr", value: "CURRENT_TIMESTAMP(3)" },
    isNullable: false,
    extra: "ON UPDATE CURRENT_TIMESTAMP(3)",
  },
];

export const DefaultTableTemplate: TableDef = {
  name: null,
  columns: DefaultDxColumns,
};

// - ADD COLUMN
// - DROP COLUMN
// - CHANGE old_col_name new_col_name data_type
//   - renaming a column
//   - changing a column's data type
// - ALTER COLUMN col SET DEFAULT literal
// - ALTER COLUMN col DROP DEFAULT
// - MODIFY COLUMN column_name data_type NULL (making a column NULL)
// - MODIFY COLUMN column_name data_type NOT NULL (making a column NOT NULL)

type ColumnMark = "Add" | "Del" | "Update" | "None";

export interface ColumnMarkMap {
  [key: number]: ColumnMark;
}

const initialState: DatabaseState = {
  schema: {},
  curWorkingTable: null,
  draftTable: { name: null, columns: [] },
  draftTableError: { nameError: null, columnsError: [] },
  draftColumnMarks: {},
  editorMod: "None",
};

type DatabaseAction =
  | { type: "LoadTables"; schemaDef: SchemaDef }
  | { type: "LoadData"; tableName: string; rows: Row[] }
  | { type: "InitDraftFromTable"; tableName: string }
  | { type: "InitDraftFromTemplate" }
  | { type: "SetDraftError"; error: TableDefError }
  | { type: "DeleteScratchTable" }
  | TableEditAction;

type TableEditAction =
  | { type: "SetTableName"; tableName: string }
  | {
      type: "AddColumn";
      column: DxColumnType;
    }
  | {
      type: "DelColumn";
      columnIndex: number;
    }
  | {
      type: "UpdateColumn";
      column: DxColumnType;
      columnIndex: number;
    };

const DatabaseStateContext = createContext<DatabaseState | null>(null);
const DatabaseDispatchContext = createContext<Dispatch<DatabaseAction> | null>(
  null
);

export function DatabaseProvider({ children }: { children: ReactNode }) {
  const [databaseState, databaseDispatch] = useImmerReducer(
    databaseReducer,
    initialState
  );
  return (
    <DatabaseStateContext.Provider value={databaseState}>
      <DatabaseDispatchContext.Provider value={databaseDispatch}>
        {children}
      </DatabaseDispatchContext.Provider>
    </DatabaseStateContext.Provider>
  );
}

export function useDatabaseState() {
  return useContext(DatabaseStateContext)!;
}

export function useDatabaseDispatch() {
  return useContext(DatabaseDispatchContext)!;
}

function databaseReducer(
  state: DatabaseState,
  action: DatabaseAction
): DatabaseState {
  switch (action.type) {
    case "LoadTables":
      state.schema = action.schemaDef;
      state.curWorkingTable = null;
      state.draftTable = { name: null, columns: [] };
      state.draftTableError = { nameError: null, columnsError: [] };
      state.draftColumnMarks = {};
      state.editorMod = "None";
      return state;
    case "LoadData":
      state.curWorkingTable = {
        tableName: action.tableName,
        rows: action.rows,
      };
      return state;
    case "InitDraftFromTable":
      const t1 = state.schema[action.tableName]!;
      state.draftTable = t1;
      state.editorMod = "Update";
      return state;
    case "InitDraftFromTemplate":
      console.log("init draft from template: ", DefaultTableTemplate);
      state.draftTable = DefaultTableTemplate;
      state.editorMod = "Create";
      return state;
    case "SetDraftError":
      state.draftTableError = action.error;
      return state;
    case "DeleteScratchTable":
      state.draftTable.name = null;
      state.draftTable.columns = [];
      state.draftTableError = { nameError: null, columnsError: [] };
      state.draftColumnMarks = {};
      state.editorMod = "None";
      return state;
    case "SetTableName":
      if (state.draftTable === null) {
        throw new Error("Cannot set table name to an empty table");
      }
      state.draftTable.name = action.tableName;
      state.draftTableError.nameError = null;
      return state;
    case "AddColumn":
      if (state.draftTable === null) {
        throw new Error("Cannot add column to an empty table");
      }

      state.draftTable.columns.push(action.column);
      state.draftColumnMarks[state.draftTable.columns.length - 1] = "Add";
      return state;
    case "DelColumn":
      if (state.draftTable === null) {
        throw new Error("Cannot drop column to an empty table");
      }
      if (state.draftColumnMarks[action.columnIndex] === "Add") {
        state.draftColumnMarks[action.columnIndex] = "None";
      } else {
        state.draftColumnMarks[action.columnIndex] = "Del";
      }
      return state;
    case "UpdateColumn":
      if (state.draftTable === null) {
        throw new Error("Cannot rename column to an empty table");
      }

      state.draftTable.columns = state.draftTable.columns.map((c, index) => {
        if (index === action.columnIndex) {
          return action.column;
        } else {
          return c;
        }
      });
      const mark = state.draftColumnMarks[action.columnIndex];
      if (mark === undefined) {
        state.draftColumnMarks[action.columnIndex] = "Update";
      }
      // ignore if the column is marked as "Add"
      return state;
  }
}

export function tableChanged(
  oldTable: TableDef | null,
  newTable: TableDef | null,
  mark: ColumnMarkMap
): boolean {
  if (oldTable?.name !== newTable?.name) {
    return true;
  }

  for (const [_, v] of Object.entries(mark)) {
    if (v !== "None") {
      return true;
    }
  }
  return false;
}
