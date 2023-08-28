import { createContext, Dispatch, ReactNode, useContext } from "react";
import { useImmerReducer } from "use-immer";

type DatabaseState = {
  schema: SchemaDef;
  curData: { tableName: string; rows: Row[] } | null;
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

export type ColumnType =
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
  columnType: ColumnType;
  defaultValue: DefaultValueType;
  isNullable: boolean;
}

const initialState: DatabaseState = {
  schema: {},
  curData: null,
};

type DatabaseAction =
  | { type: "LoadTables"; schemaDef: SchemaDef }
  | { type: "LoadData"; tableName: string; rows: Row[] };

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
      state.curData = { tableName: action.tableName, rows: action.rows };
      return state;
  }
}
