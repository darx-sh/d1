import { createContext, Dispatch, ReactNode, useContext } from "react";
import { enableMapSet } from "immer";
import { useImmerReducer } from "use-immer";

enableMapSet();

type DatabaseState = {
  schema: SchemaDef;
  curDisplayData: { tableName: string; rows: Row[] } | null;
  // table editor's state
  draftTable: TableDef;
  draftTableError: TableDefError;
  draftColumnMarks: ColumnMarkMap;
  isCreateTable: boolean;
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

export type DxFieldType = "int64" | "float64" | "bool" | "datetime" | "text";

export type MySQLFieldType =
  // numeric data types
  // | "tinyint"
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
  // | "varchar"
  // | "binary"
  // | "varbinary"
  // | "blob"
  | "text";
// | "enum"
// | "set"
// json
// | "json";

export type ColumnTypeMap = { [K in MySQLFieldType]: DxFieldType };

// MySQL types -> Darx types.
export const columnTypesMap: ColumnTypeMap = {
  // tinyint: "tinyint",
  // smallint: "smallint",
  // mediumint: "mediumint",
  // int: "int",
  bigint: "int64",
  // decimal: "decimal",
  // numeric: "numeric",
  // float: "float",
  double: "float64",
  // bit: "bit",
  // date: "date",
  // time: "time",
  datetime: "datetime",
  // timestamp: "timestamp",
  // year: "year",
  // char: "char",
  // varchar: "varchar",
  // binary: "binary",
  // varbinary: "varbinary",
  // blob: "blob",
  text: "text",
  // enum: "enum",
  // set: "set",
  // json: "json",
};

export function getAllColumnTypes(): DxFieldType[] {
  return Object.values(columnTypesMap);
}

export type DefaultValueType = null | string | number | boolean;

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
  }
}

export interface DxColumnType {
  name: string | null;
  fieldType: DxFieldType | null;
  defaultValue: DefaultValueType;
  isNullable: boolean;
  extra: ExtraColumnOptions | null;
}

type ExtraColumnOptions = "AUTO_INCREMENT" | "ON UPDATE CURRENT_TIMESTAMP(3)";

export const DefaultDxColumns: DxColumnType[] = [
  {
    name: "id",
    fieldType: "int64",
    defaultValue: 0,
    isNullable: false,
    extra: "AUTO_INCREMENT",
  },
  {
    name: "created_at",
    fieldType: "datetime",
    defaultValue: "CURRENT_TIMESTAMP(3)",
    isNullable: false,
    extra: null,
  },
  {
    name: "updated_at",
    fieldType: "datetime",
    defaultValue: "CURRENT_TIMESTAMP(3)",
    isNullable: false,
    extra: "ON UPDATE CURRENT_TIMESTAMP(3)",
  },
];

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
  columns: DxColumnType[];
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
  draftTable: { name: null, columns: [] },
  draftTableError: { nameError: null, columnsError: [] },
  draftColumnMarks: {},
  isCreateTable: true,
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
      return state;
    case "LoadData":
      state.curDisplayData = { tableName: action.tableName, rows: action.rows };
      return state;
    case "InitDraftFromTable":
      const t1 = state.schema[action.tableName]!;
      state.draftTable = t1;
      state.isCreateTable = false;
      return state;
    case "InitDraftFromTemplate":
      const t2: TableDef = {
        name: null,
        columns: DefaultDxColumns,
      };
      state.draftTable = t2;
      state.isCreateTable = true;
      return state;
    case "SetDraftError":
      state.draftTableError = action.error;
      return state;
    case "DeleteScratchTable":
      state.draftTable.name = null;
      state.draftTable.columns = [];
      state.draftTableError = { nameError: null, columnsError: [] };
      state.isCreateTable = true;
      return state;
    // case "CreateTable":
    //   if (state.draftTable !== null) {
    //     throw new Error("Cannot create table while there is a table");
    //   }
    //   state.draftTable = action.payload;
    //   return state;
    // case "RenameTable":
    //   if (state.draftTable === null) {
    //     throw new Error("Cannot rename an empty table");
    //   }
    //
    //   if (state.draftTable.name !== action.oldTableName) {
    //     const scratchTableName = state.draftTable.name ?? "null";
    //     throw new Error(
    //       `Invalid oldTableName: action = ${action.oldTableName} scratch = ${scratchTableName}`
    //     );
    //   }
    //
    //   state.draftTable.name = action.newTableName;
    //   return state;
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
      state.draftColumnMarks[action.columnIndex] = "Del";
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
      return state;
  }
}
