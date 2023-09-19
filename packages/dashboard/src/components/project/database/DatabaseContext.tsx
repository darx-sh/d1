import { createContext, Dispatch, ReactNode, useContext } from "react";
import { enableMapSet } from "immer";
import { useImmerReducer } from "use-immer";
import { FieldType, Datum, DatumNotNull, DefaultValue } from "~/utils/types";

enableMapSet();

type DatabaseState = {
  schema: SchemaDef;
  curNav: NavDef;
  // schema editor's state
  draftTable: TableDef;
  draftTableError: TableDefError;
  schemaActions: SchemaActionMap;
  editorMode: "Create" | "Update" | "None";
  draftOriginalTable: string | null;

  // Row editor's state
  rowEditorColumnActions: RowColumnActionMap;
  rowEditorMode: RowEditorMode;
  rowEditorOriginalRow: Row;
};

export type RowEditorMode = "Create" | "Update" | "None";

export type NavDef = { typ: "Schema" } | { typ: "Table"; tableName: string };

export interface Row {
  [key: string]: Datum;
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

export function isSystemField(column_name: string) {
  return ["id", "created_at", "updated_at"].includes(column_name);
}

const DefaultDxColumns: DxColumnType[] = [
  {
    name: "id",
    fieldType: "int64 Auto Increment",
    defaultValue: { typ: "Not Defined" },
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

type SchemaAction = "Add" | "Del" | "Update" | "None";

export interface SchemaActionMap {
  [key: number]: SchemaAction;
}

// column's modification action of a row.
// it is used for insert/update a row.
type ColumnAction =
  | { typ: "SetDefault" }
  | { typ: "SetNull" }
  | { typ: "SetRegular"; value: DatumNotNull };

interface RowColumnActionMap {
  [key: string]: ColumnAction;
}

const initialState: DatabaseState = {
  schema: {},
  curNav: { typ: "Schema" },
  draftTable: { name: null, columns: [] },
  draftTableError: { nameError: null, columnsError: [] },
  schemaActions: {},
  editorMode: "None",
  draftOriginalTable: null,
  rowEditorColumnActions: {},
  rowEditorMode: "None",
  rowEditorOriginalRow: {},
};

type DatabaseAction =
  | { type: "LoadSchema"; schemaDef: SchemaDef }
  | { type: "SetNav"; nav: NavDef }
  | { type: "InitDraftFromTable"; tableName: string }
  | { type: "InitDraftFromTemplate" }
  | { type: "SetDraftError"; error: TableDefError }
  | { type: "DeleteScratchTable" }
  | TableEditAction
  | { type: "InitRowEditorFromEmpty" }
  | { type: "InitRowEditorFromRow"; row: Row }
  | { type: "DeleteRowEditor" }
  | { type: "SetColumnAction"; columnName: string; columnAction: ColumnAction };

type TableEditAction =
  | { type: "SetTableName"; tableName: string }
  | {
      type: "SchemaAddColumn";
      column: DxColumnType;
    }
  | {
      type: "SchemaDelColumn";
      columnIndex: number;
    }
  | {
      type: "SchemaUpdateColumn";
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
    case "LoadSchema":
      state.schema = action.schemaDef;
      state.draftTable = { name: null, columns: [] };
      state.draftTableError = { nameError: null, columnsError: [] };
      state.schemaActions = {};
      state.editorMode = "None";
      state.draftOriginalTable = null;
      return state;
    case "SetNav":
      state.curNav = action.nav;
      return state;
    case "InitDraftFromTable":
      const t1 = state.schema[action.tableName]!;
      state.draftTable = t1;
      state.editorMode = "Update";
      state.draftOriginalTable = action.tableName;
      return state;
    case "InitDraftFromTemplate":
      state.draftTable = DefaultTableTemplate;
      state.editorMode = "Create";
      state.draftOriginalTable = null;
      return state;
    case "SetDraftError":
      state.draftTableError = action.error;
      return state;
    case "DeleteScratchTable":
      state.draftTable.name = null;
      state.draftTable.columns = [];
      state.draftTableError = { nameError: null, columnsError: [] };
      state.schemaActions = {};
      state.editorMode = "None";
      state.draftOriginalTable = null;
      return state;
    // TableEditAction...
    case "SetTableName":
      if (state.draftTable === null) {
        throw new Error("Cannot set table name to an empty table");
      }
      state.draftTable.name = action.tableName;
      state.draftTableError.nameError = null;
      return state;
    case "SchemaAddColumn":
      if (state.draftTable === null) {
        throw new Error("Cannot add column to an empty table");
      }

      state.draftTable.columns.push(action.column);
      state.schemaActions[state.draftTable.columns.length - 1] = "Add";
      return state;
    case "SchemaDelColumn":
      if (state.draftTable === null) {
        throw new Error("Cannot drop column to an empty table");
      }
      if (state.schemaActions[action.columnIndex] === "Add") {
        state.schemaActions[action.columnIndex] = "None";
      } else {
        state.schemaActions[action.columnIndex] = "Del";
      }
      return state;
    case "SchemaUpdateColumn":
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
      const mark = state.schemaActions[action.columnIndex];
      if (mark === undefined) {
        state.schemaActions[action.columnIndex] = "Update";
      }
      // ignore if the column is marked as "Add"
      return state;
    // TableEditAction end
    case "InitRowEditorFromEmpty":
      state.rowEditorColumnActions = {};
      state.rowEditorMode = "Create";
      state.rowEditorOriginalRow = {};
      return state;
    case "InitRowEditorFromRow":
      state.rowEditorColumnActions = {};
      state.rowEditorMode = "Update";
      state.rowEditorOriginalRow = action.row;
      return state;
    case "DeleteRowEditor":
      state.rowEditorColumnActions = {};
      state.rowEditorOriginalRow = {};
      state.rowEditorMode = "None";
      return state;
    case "SetColumnAction":
      state.rowEditorColumnActions[action.columnName] = action.columnAction;
      return state;
  }
}

export type RowEditorColumnValue =
  | { typ: "Not Defined" }
  | { typ: "NULL" }
  | { typ: "Default" }
  | DatumNotNull;

function rowEditorNewColumnValue(
  columnActions: RowColumnActionMap,
  columnName: string
): RowEditorColumnValue {
  const action = columnActions[columnName];
  if (action === undefined) {
    return { typ: "Not Defined" };
  } else {
    switch (action.typ) {
      case "SetDefault":
        return { typ: "Default" };
      case "SetNull":
        return { typ: "NULL" };
      case "SetRegular":
        return action.value;
    }
  }
}

function rowEditorUpdateColumnValue(
  columnActions: RowColumnActionMap,
  columnName: string,
  originalRow: Row
): RowEditorColumnValue {
  const action = columnActions[columnName];
  if (action === undefined) {
    return originalRow[columnName]!;
  } else {
    switch (action.typ) {
      case "SetDefault":
        return { typ: "Default" };
      case "SetNull":
        return { typ: "NULL" };
      case "SetRegular":
        return action.value;
    }
  }
}

export function rowEditorColumnValue(
  state: DatabaseState,
  columnType: DxColumnType
): RowEditorColumnValue {
  const columnName = columnType.name;
  switch (state.rowEditorMode) {
    case "Create":
      return rowEditorNewColumnValue(state.rowEditorColumnActions, columnName);
    case "Update":
      return rowEditorUpdateColumnValue(
        state.rowEditorColumnActions,
        columnName,
        state.rowEditorOriginalRow
      );
    case "None":
      return { typ: "Not Defined" };
  }
}

export function getInsertRow(state: DatabaseState): Row {
  const { changes, deletedColumns } = rowActionToRow(
    state.rowEditorColumnActions
  );
  const insert = { ...state.rowEditorOriginalRow, ...changes };
  deletedColumns.forEach((c) => {
    if (insert[c] !== undefined) delete insert[c];
  });
  return insert;
}

export function getUpdateRow(state: DatabaseState): Row {
  const { changes } = rowActionToRow(state.rowEditorColumnActions);
  return changes;
}

export function tableChanged(
  oldTable: TableDef | null,
  newTable: TableDef | null,
  mark: SchemaActionMap
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

function rowActionToRow(actions: RowColumnActionMap) {
  const changes: Row = {};
  const deletedColumns: Set<string> = new Set();
  Object.entries(actions).map(([k, v]) => {
    switch (v.typ) {
      case "SetNull":
        changes[k] = { typ: "NULL" };
        break;
      case "SetDefault":
        deletedColumns.add(k);
        break;
      case "SetRegular":
        changes[k] = v.value;
        break;
    }
  });
  return { changes, deletedColumns };
}
