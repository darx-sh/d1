import { createContext, Dispatch, ReactNode, useContext } from "react";
import { enableMapSet } from "immer";
import { useImmerReducer } from "use-immer";

enableMapSet();

type DatabaseState = {
  schema: SchemaDef;
  curDisplayData: { tableName: string; rows: Row[] } | null;
  scratchTable: TableDef | null;
  deletedScratchColumns: Set<number>;
  ddlLogs: DDLLog[];
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
  name: string;
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
  name: string;
  fieldType: FieldType;
  defaultValue: DefaultValueType;
  isNullable: boolean;
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

interface AddColumn {
  tableName: string;
  column: ColumnDef;
}

interface DropColumn {
  tableName: string;
  columnName: string;
}

interface RenameColumn {
  tableName: string;
  oldColumnName: string;
  newColumnName: string;
}

const initialState: DatabaseState = {
  schema: {},
  curDisplayData: null,
  scratchTable: null,
  deletedScratchColumns: new Set<number>(),
  ddlLogs: [],
};

type DatabaseAction =
  | { type: "LoadTables"; schemaDef: SchemaDef }
  | { type: "LoadData"; tableName: string; rows: Row[] }
  | { type: "InitScratchTable"; payload: TableDef }
  | { type: "DeleteScratchTable" }
  | { type: "CreateTable"; payload: CreateTable }
  | { type: "DropTable"; payload: DropTable }
  | TableEditAction;

// todo: MODIFY COLUMN
type TableEditAction =
  | { type: "RenameTable"; payload: RenameTable; prepareDDL: boolean }
  | {
      type: "AddColumn";
      payload: AddColumn;
      columnIndex: number;
      prepareDDL: boolean;
    }
  | {
      type: "DropColumn";
      payload: DropColumn;
      columnIndex: number;
      prepareDDL: boolean;
    }
  | {
      type: "RenameColumn";
      payload: RenameColumn;
      columnIndex: number;
      prepareDDL: boolean;
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
      state.scratchTable = null;
      return state;
    case "CreateTable":
      if (state.ddlLogs.length !== 0) {
        throw new Error("Cannot create table while there are pending DDLs");
      }
      state.ddlLogs.push({ type: "CreateTable", payload: action.payload });
      return state;
    case "DropTable":
      if (state.ddlLogs.length !== 0) {
        throw new Error("Cannot drop table while there are pending DDLs");
      }
      state.ddlLogs.push({ type: "DropTable", payload: action.payload });
      return state;
    case "RenameTable":
      if (state.scratchTable === null) {
        throw new Error("Cannot rename an empty table");
      }

      if (state.scratchTable.name !== action.payload.oldTableName) {
        throw new Error(
          `Invalid oldTableName: action = ${action.payload.oldTableName} scratch = ${state.scratchTable.name}`
        );
      }

      state.scratchTable.name = action.payload.newTableName;
      if (action.prepareDDL) {
        state.ddlLogs.push({ type: "RenameTable", payload: action.payload });
      }
      return state;
    case "AddColumn":
      if (state.scratchTable === null) {
        throw new Error("Cannot add column to an empty table");
      }

      state.scratchTable.columns.push(action.payload.column);
      if (action.prepareDDL) {
        state.ddlLogs.push({
          type: "AddColumn",
          payload: { index: action.columnIndex },
        });
      }
      return state;
    case "DropColumn":
      if (state.scratchTable === null) {
        throw new Error("Cannot drop column to an empty table");
      }
      state.deletedScratchColumns.add(action.columnIndex);
      if (action.prepareDDL) {
        state.ddlLogs.push({
          type: "DropColumn",
          payload: { index: action.columnIndex },
        });
      }
      return state;
    case "RenameColumn":
      if (state.scratchTable === null) {
        throw new Error("Cannot rename column to an empty table");
      }

      state.scratchTable.columns = state.scratchTable.columns.map((c) => {
        if (c.name === action.payload.oldColumnName) {
          return { ...c, name: action.payload.newColumnName };
        } else {
          return c;
        }
      });
      if (action.prepareDDL) {
        state.ddlLogs.push({
          type: "RenameColumn",
          payload: { index: action.columnIndex },
        });
      }
      return state;
  }
}
