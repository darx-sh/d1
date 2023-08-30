import { createContext, Dispatch, ReactNode, useContext } from "react";
import { enableMapSet } from "immer";
import { useImmerReducer } from "use-immer";

enableMapSet();

type DatabaseState = {
  schema: SchemaDef;
  curDisplayData: { tableName: string; rows: Row[] } | null;
  scratchTable: TableDef;
  columnMarks: ColumnMarkMap;
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
  columns: ColumnDef[];
}

export type FieldType =
  // numeric data types
  | "tinyint"
  | "smallint"
  | "mediumint"
  | "int"
  | "bigint"
  | "decimal"
  | "numeric"
  | "float"
  | "double"
  | "bit"
  // date and time data types
  | "date"
  | "time"
  | "datetime"
  | "timestamp"
  | "year"
  // string data types
  | "char"
  | "varchar"
  | "binary"
  | "varbinary"
  | "blob"
  | "text"
  | "enum"
  | "set"
  // json
  | "json";

type ColumnTypeMap = { [K in FieldType]: K };

const columnTypes: ColumnTypeMap = {
  tinyint: "tinyint",
  smallint: "smallint",
  mediumint: "mediumint",
  int: "int",
  bigint: "bigint",
  decimal: "decimal",
  numeric: "numeric",
  float: "float",
  double: "double",
  bit: "bit",
  date: "date",
  time: "time",
  datetime: "datetime",
  timestamp: "timestamp",
  year: "year",
  char: "char",
  varchar: "varchar",
  binary: "binary",
  varbinary: "varbinary",
  blob: "blob",
  text: "text",
  enum: "enum",
  set: "set",
  json: "json",
};

export function getAllColumnTypes(): FieldType[] {
  return Object.values(columnTypes);
}

export type DefaultValueType = null | string | number | boolean | object;

export function displayDefaultValue(v: DefaultValueType) {
  if (v === null) {
    return "NULL";
  }
  const t = typeof v;
  if (t === "string") {
    return `${v as string}`;
  } else if (t === "number") {
    return (v as number).toString();
  } else if (t === "boolean") {
    return (v as boolean).toString();
  } else if (t === "object") {
    return JSON.stringify(v);
  }
}

export interface ColumnDef {
  name: string | null;
  fieldType: FieldType | null;
  defaultValue: DefaultValueType;
  isNullable: boolean;
  isPrimary: boolean;
}

// - ADD COLUMN
// - DROP COLUMN
// - CHANGE old_col_name new_col_name data_type
//   - renaming a column
//   - changing a column's data type
// - ALTER COLUMN col SET DEFAULT literal
// - ALTER COLUMN col DROP DEFAULT
// - MODIFY COLUMN column_name data_type NULL (making a column NULL)
// - MODIFY COLUMN column_name data_type NOT NULL (making a column NOT NULL)

type ColumnMark = "Add" | "Del" | "Update";

interface ColumnMarkMap {
  [key: number]: ColumnMark;
}

type DDLLog =
  | { type: "CreateTable"; payload: CreateTable }
  | { type: "DropTable"; payload: DropTable }
  | { type: "RenameTable"; payload: RenameTable }
  | { type: "AddColumn"; payload: { index: number } }
  | { type: "DropColumn"; payload: { index: number } }
  | { type: "RenameColumn"; payload: { index: number } };

type DDLAction =
  | { type: "CreateTable"; payload: CreateTable }
  | { type: "DropTable"; payload: DropTable }
  | TableEditAction;

interface CreateTable {
  tableName: string;
  columns: ColumnDef[];
}

interface DropTable {
  tableName: string;
}

interface RenameTable {
  oldTableName: string;
  newTableName: string;
}

const initialState: DatabaseState = {
  schema: {},
  curDisplayData: null,
  scratchTable: { name: null, columns: [] },
  columnMarks: {},
};

type DatabaseAction =
  | { type: "LoadTables"; schemaDef: SchemaDef }
  | { type: "LoadData"; tableName: string; rows: Row[] }
  | { type: "InitScratchTable"; payload: TableDef }
  | { type: "DeleteScratchTable" }
  | { type: "CreateTable"; payload: TableDef }
  | TableEditAction;

// todo: MODIFY COLUMN
type TableEditAction =
  | { type: "RenameTable"; oldTableName: string; newTableName: string }
  | {
      type: "AddColumn";
      column: ColumnDef;
    }
  | {
      type: "DelColumn";
      columnIndex: number;
    }
  | {
      type: "UpdateColumn";
      column: ColumnDef;
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
      return state;
    case "LoadData":
      state.curDisplayData = { tableName: action.tableName, rows: action.rows };
      return state;
    case "InitScratchTable":
      state.scratchTable = action.payload;
      return state;
    case "DeleteScratchTable":
      state.scratchTable.name = null;
      state.scratchTable.columns = [];
      return state;
    case "CreateTable":
      if (state.scratchTable !== null) {
        throw new Error("Cannot create table while there is a table");
      }
      state.scratchTable = action.payload;
      return state;
    case "RenameTable":
      if (state.scratchTable === null) {
        throw new Error("Cannot rename an empty table");
      }

      if (state.scratchTable.name !== action.oldTableName) {
        const scratchTableName = state.scratchTable.name ?? "null";
        throw new Error(
          `Invalid oldTableName: action = ${action.oldTableName} scratch = ${scratchTableName}`
        );
      }

      state.scratchTable.name = action.newTableName;
      return state;
    case "AddColumn":
      if (state.scratchTable === null) {
        throw new Error("Cannot add column to an empty table");
      }

      state.scratchTable.columns.push(action.column);
      state.columnMarks[state.scratchTable.columns.length - 1] = "Add";
      return state;
    case "DelColumn":
      if (state.scratchTable === null) {
        throw new Error("Cannot drop column to an empty table");
      }
      state.columnMarks[action.columnIndex] = "Del";
      return state;
    case "UpdateColumn":
      if (state.scratchTable === null) {
        throw new Error("Cannot rename column to an empty table");
      }

      state.scratchTable.columns = state.scratchTable.columns.map(
        (c, index) => {
          if (index === action.columnIndex) {
            return action.column;
          } else {
            return c;
          }
        }
      );
      const mark = state.columnMarks[action.columnIndex];
      if (mark === undefined) {
        state.columnMarks[action.columnIndex] = "Update";
      }
      return state;
  }
}
