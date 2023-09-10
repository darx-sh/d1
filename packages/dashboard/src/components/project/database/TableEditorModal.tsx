import { Fragment, useRef, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon, ExclamationCircleIcon } from "@heroicons/react/24/outline";
import ColumnsEditor from "~/components/project/database/ColumnsEditor";
import {
  useDatabaseDispatch,
  useDatabaseState,
} from "~/components/project/database/DatabaseContext";
import {
  ColumnError,
  ColumnMarkMap,
  DefaultTableTemplate,
  tableChanged,
  TableDef,
  TableDefError,
} from "~/components/project/database/DatabaseContext";
import {
  ddl,
  loadSchema,
  CreateTableReq,
  TableEditReq,
} from "~/components/project/database/Api";
import DangerActionConfirm from "~/components/project/database/DangerActionConfirm";

type TableEditorProps = {
  envId: string;
  open: boolean;
  beforeSave: () => void;
  afterSave: () => void;
};

export default function TableEditorModal(props: TableEditorProps) {
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();
  const { open } = props;
  const tableDef = state.draftTable;
  const tableDefError = state.draftTableError;
  const tableNameRef = useRef<HTMLInputElement>(null);
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);

  const handleSave = async () => {
    const mod = state.editorMod;
    if (mod === "Create") {
      props.beforeSave();
      await createTable();
    } else if (mod === "Update") {
      props.beforeSave();
      await updateTable();
    }
    props.afterSave();
  };

  const handleCancel = () => {
    const marks = state.draftColumnMarks;
    let hasChanged = false;
    switch (state.editorMod) {
      case "Create":
        if (tableChanged(DefaultTableTemplate, state.draftTable, marks)) {
          hasChanged = true;
        }
        break;
      case "Update":
        const tableName = state.draftOriginalTable!;
        if (tableChanged(state.schema[tableName]!, state.draftTable, marks)) {
          hasChanged = true;
        }
        break;
      case "None":
        throw new Error("cancel editor in non-edit mode");
    }

    if (hasChanged) {
      setShowCancelConfirm(true);
      return;
    } else {
      dispatch({ type: "DeleteScratchTable" });
      return;
    }
  };

  const createTable = async () => {
    const tableDef = state.draftTable;
    const error = validateTableDef(tableDef);
    if (error !== null) {
      dispatch({ type: "SetDraftError", error });
    } else {
      const req = genCreateTable(tableDef);
      const rsp = await ddl(props.envId, req);
      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
    }
  };

  const updateTable = async () => {
    const curTableName = state.draftOriginalTable!;
    const oldTableDef = state.schema[curTableName]!;
    const newTableDef = state.draftTable;
    const marks = state.draftColumnMarks;
    const error = validateTableDef(newTableDef);
    if (error !== null) {
      dispatch({ type: "SetDraftError", error });
    } else {
      const reqs = genTableEdit(oldTableDef, newTableDef, marks);
      for (const req of reqs) {
        const rsp = await ddl(props.envId, req);
      }
      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
    }
  };

  const renderContent = () => {
    return (
      <>
        <Transition.Root show={open} as={Fragment}>
          <Dialog
            as="div"
            className="relative z-10"
            onClose={handleCancel}
            initialFocus={
              state.editorMod === "Create" ? tableNameRef : undefined
            }
          >
            <Transition.Child
              as={Fragment}
              enter="ease-in-out duration-250"
              enterFrom="opacity-0"
              enterTo="opacity-100"
              leave="ease-in-out duration-250"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" />
            </Transition.Child>

            <div className="fixed inset-0 overflow-hidden">
              <div className="absolute inset-0 overflow-hidden">
                <div className="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10 sm:pl-16">
                  <Transition.Child
                    as={Fragment}
                    enter="transform transition ease-in-out duration-250"
                    enterFrom="translate-x-full"
                    enterTo="translate-x-0"
                    leave="transform transition ease-in-out duration-250"
                    leaveFrom="translate-x-0"
                    leaveTo="translate-x-full"
                  >
                    <Dialog.Panel className="pointer-events-auto w-screen max-w-2xl">
                      <div className="flex h-full flex-col overflow-y-scroll bg-white py-6 shadow-xl">
                        <div className="px-4 sm:px-6">
                          <div className="flex items-start justify-between">
                            <Dialog.Title className="text-base font-semibold leading-6 text-gray-900">
                              Table Editor
                            </Dialog.Title>
                            <div className="ml-3 flex h-7 items-center">
                              <button
                                type="button"
                                className="relative rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
                                onClick={handleCancel}
                              >
                                <span className="absolute -inset-2.5" />
                                <span className="sr-only">Close panel</span>
                                <XMarkIcon
                                  className="h-6 w-6"
                                  aria-hidden="true"
                                />
                              </button>
                            </div>
                          </div>
                        </div>
                        <div className="relative flex-1 px-4">
                          <form>
                            <div className="space-y-6">
                              <div className="border-b border-gray-900/10 pb-6">
                                <div className="mt-10 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6">
                                  <div className="sm:col-span-4">
                                    <label
                                      htmlFor="tableName"
                                      className="block text-sm font-medium leading-6 text-gray-900"
                                    >
                                      Table Name
                                    </label>
                                    <div className="relative mt-2 flex rounded-md shadow-sm">
                                      <input
                                        ref={tableNameRef}
                                        defaultValue={tableDef.name ?? ""}
                                        type="text"
                                        name="tableName"
                                        id="tableName"
                                        autoComplete="tableName"
                                        className={
                                          tableDefError.nameError === null
                                            ? "block rounded-md border-0 px-1.5 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
                                            : "block rounded-md border-0 px-1.5 py-1.5 text-red-900 shadow-sm ring-1 ring-inset ring-red-300 placeholder:text-red-400 focus:ring-2 focus:ring-inset focus:ring-red-600"
                                        }
                                        onChange={(event) => {
                                          const v = event.target.value;
                                          dispatch({
                                            type: "SetTableName",
                                            tableName: v,
                                          });
                                        }}
                                      />
                                      {tableDefError.nameError ===
                                      null ? null : (
                                        <>
                                          <div className="pointer-events-none inset-y-0 right-0 -ml-6 flex items-center pr-3">
                                            <ExclamationCircleIcon
                                              className="h-5 w-5 text-red-500"
                                              aria-hidden="true"
                                            />
                                          </div>
                                          <p
                                            className="mt-2 text-sm text-red-600"
                                            id="tableName-error"
                                          >
                                            Not a valid table name
                                          </p>
                                        </>
                                      )}
                                    </div>
                                  </div>
                                </div>
                              </div>

                              <div className="border-b border-gray-900/10 pb-12">
                                <h2 className="text-base font-normal leading-7 text-gray-900">
                                  Columns
                                </h2>
                                <ColumnsEditor></ColumnsEditor>
                              </div>
                            </div>

                            <div className="mt-6 flex items-center justify-end gap-x-6">
                              <button
                                type="button"
                                className="text-sm font-semibold leading-6 text-gray-900"
                                onClick={() => {
                                  handleCancel();
                                }}
                              >
                                Cancel
                              </button>
                              <button
                                type="button"
                                className="rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                                onClick={handleSave}
                              >
                                Save
                              </button>
                            </div>
                          </form>
                        </div>
                      </div>
                    </Dialog.Panel>
                  </Transition.Child>
                </div>
              </div>
            </div>
          </Dialog>
        </Transition.Root>
        <DangerActionConfirm
          open={showCancelConfirm}
          message={
            "There is unsaved changes. Do you want to discard the changes?"
          }
          onYes={() => {
            setShowCancelConfirm(false);
            dispatch({ type: "DeleteScratchTable" });
          }}
          onNo={() => {
            setShowCancelConfirm(false);
          }}
        ></DangerActionConfirm>
      </>
    );
  };
  return renderContent();
}

function validateTableDef(tableDef: TableDef): TableDefError | null {
  let hasError = false;
  const tableDefErr: TableDefError = { nameError: null, columnsError: [] };

  if (tableDef.name === null || tableDef.name === "") {
    hasError = true;
    tableDefErr.nameError = "Table name cannot be empty";
  }

  tableDef.columns.forEach((col) => {
    const columnError: ColumnError = {
      nameError: null,
      fieldTypeError: null,
    };
    if (col.name === "") {
      hasError = true;
      columnError.nameError = "Column name cannot be empty";
    }
    if (col.fieldType === "NotDefined") {
      hasError = true;
      columnError.fieldTypeError = "Column type cannot be empty";
    }

    // rule 1: text field cannot have default value

    // rule 2: AUTO_INCREMENT cannot have a default value

    // rule 3: AUTO_INCREMENT cannot be nullable

    // rule 4: AUTO_INCREMENT MUST be a KEY.
    tableDefErr.columnsError.push(columnError);
  });

  if (hasError) {
    return tableDefErr;
  } else {
    return null;
  }
}

function genCreateTable(tableDef: TableDef): CreateTableReq {
  const columns = tableDef.columns.map((c) => {
    const name = c.name;
    const fieldType = c.fieldType;
    const isNullable = c.isNullable;
    const extra = c.extra;
    return {
      name,
      fieldType,
      isNullable,
      defaultValue: c.defaultValue,
      extra,
    };
  });
  const req = {
    createTable: {
      tableName: tableDef.name!,
      columns: columns,
    },
  };
  return req;
}

function genTableEdit(
  oldTable: TableDef,
  newTable: TableDef,
  marks: ColumnMarkMap
): TableEditReq[] {
  const reqs: TableEditReq[] = [];
  if (oldTable.name! !== newTable.name!) {
    reqs.push({
      renameTable: {
        oldTableName: oldTable.name!,
        newTableName: newTable.name!,
      },
    });
  }

  console.assert(
    newTable.columns.length >= oldTable.columns.length,
    "new table should have more columns than old table"
  );

  for (let i = 0; i < newTable.columns.length; i++) {
    // handle "add", "delete", "update"; ignore none.
    switch (marks[i]) {
      case undefined:
        break;
      case "None":
        break;
      case "Add":
        reqs.push({
          addColumn: {
            tableName: newTable.name!,
            column: newTable.columns[i]!,
          },
        });
        break;
      case "Update":
        // Can only rename column name now.
        // Other properties like column type, default value, nullable cannot be changed after table is created.
        if (oldTable.columns[i]!.name !== newTable.columns[i]!.name) {
          reqs.push({
            renameColumn: {
              tableName: newTable.name!,
              oldColumnName: oldTable.columns[i]!.name,
              newColumnName: newTable.columns[i]!.name,
            },
          });
        }
        break;
      case "Del":
        reqs.push({
          dropColumn: {
            tableName: newTable.name!,
            columnName: newTable.columns[i]!.name,
          },
        });
        break;
    }
  }
  return reqs;
}
