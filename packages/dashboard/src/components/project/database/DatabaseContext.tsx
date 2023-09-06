import { createContext, Dispatch, ReactNode, useContext } from "react";
import { enableMapSet } from "immer";
import { useImmerReducer } from "use-immer";
import { FieldType, DefaultValue } from "~/utils/types";

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

export interface DxColumnType {
  name: string;
  fieldType: FieldType;
  defaultValue: DefaultValue;
  isNullable: boolean;
  extra: ExtraColumnOptions | null;
}

type ExtraColumnOptions = "AUTO_INCREMENT" | "ON UPDATE CURRENT_TIMESTAMP(3)";

const DefaultDxColumns: DxColumnType[] = [
  {
    name: "id",
    fieldType: "int64Identity",
    defaultValue: { typ: "NotDefined", value: "" },
    isNullable: false,
    extra: "AUTO_INCREMENT",
  },
  {
    name: "created_at",
    fieldType: "datetime",
    defaultValue: { typ: "expr", value: "CURRENT_TIMESTAMP(3)" },
    isNullable: false,
    extra: null,
  },
  {
    name: "updated_at",
    fieldType: "datetime",
    defaultValue: { typ: "expr", value: "CURRENT_TIMESTAMP(3)" },
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
