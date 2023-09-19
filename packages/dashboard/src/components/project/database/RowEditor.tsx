import { ChangeEvent, Fragment, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import DangerActionConfirm from "~/components/project/database/DangerActionConfirm";
import {
  useDatabaseState,
  useDatabaseDispatch,
  getInsertRow,
  getUpdateRow,
  DxColumnType,
  rowEditorColumnValue,
  isSystemField,
  RowEditorColumnValue,
} from "~/components/project/database/DatabaseContext";
import {
  PrimitiveType,
  displayDefaultValue,
  displayDatum,
  datumToPrimitive,
} from "~/utils/types";
import {
  insertRow,
  RowDatumToApiRow,
  updateRow,
} from "~/components/project/database/Api";
import { DateTimePicker } from "@mui/x-date-pickers/DateTimePicker";
import { Dayjs } from "dayjs";
import dayjs from "dayjs";

interface ColumnProps {
  columnType: DxColumnType;
}

function ColumnDatetime(props: ColumnProps) {
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();
  const columnType = props.columnType;
  const value = rowEditorColumnValue(state, columnType);

  switch (value.typ) {
    case "Default":
      return (
        <span className="bg-gray-200">
          {displayDefaultValue(columnType.defaultValue)}
        </span>
      );
    case "NULL":
    case "Not Defined":
      return (
        <DateTimePicker
          ampm={false}
          defaultValue={null}
          onChange={(value: Dayjs | null, _context) => {
            if (value !== null) {
              const dt = value.format("2021-06-30 13:03:47.123Z");
              dispatch({
                type: "SetColumnAction",
                columnName: props.columnType.name,
                columnAction: {
                  typ: "SetRegular",
                  value: { typ: "datetime", value: dt },
                },
              });
            }
          }}
        ></DateTimePicker>
      );
    default:
      const v = value.value as string;
      const defaultDt = dayjs(v);
      return (
        <DateTimePicker
          ampm={false}
          defaultValue={defaultDt}
          onChange={(value: Dayjs | null, _context) => {
            if (value !== null) {
              const dt = value.format("2021-06-30 13:03:47.123Z");
              dispatch({
                type: "SetColumnAction",
                columnName: props.columnType.name,
                columnAction: {
                  typ: "SetRegular",
                  value: { typ: "datetime", value: dt },
                },
              });
            }
          }}
        ></DateTimePicker>
      );
  }
}

function ColumnNumber(props: ColumnProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const columnType = props.columnType;
  const value = rowEditorColumnValue(state, columnType);

  const columnValue = () => {
    switch (value.typ) {
      case "Not Defined":
      case "NULL":
        return "";
      case "Default":
        const v = columnType.defaultValue as { typ: "int64"; value: number };
        return v.value.toString();
      default:
        const v2 = value as { typ: "int64"; value: number };
        return v2.value.toString();
    }
  };

  return (
    <input
      type="number"
      value={columnValue()}
      className="ring-inset-0 mt-2 rounded p-2 text-sm text-gray-900 ring-1 ring-gray-300"
      onChange={(e) => {
        const value = e.target.value;
        dispatch({
          type: "SetColumnAction",
          columnName: columnType.name,
          columnAction: {
            typ: "SetRegular",
            value: { typ: "int64", value: parseInt(value) },
          },
        });
      }}
    ></input>
  );
}

function ColumnBool(props: ColumnProps) {
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();
  const columnType = props.columnType;
  const name = columnType.name;
  const value = rowEditorColumnValue(state, columnType);

  const handleOptionChange = (e: ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    if (v === "true") {
      dispatch({
        type: "SetColumnAction",
        columnName: name,
        columnAction: {
          typ: "SetRegular",
          value: { typ: "bool", value: true },
        },
      });
    } else if (v === "false") {
      dispatch({
        type: "SetColumnAction",
        columnName: name,
        columnAction: {
          typ: "SetRegular",
          value: { typ: "bool", value: false },
        },
      });
    } else {
      throw new Error("unknown value: " + v);
    }
  };

  return (
    <fieldset className="flex space-x-6">
      <div className="flex items-center space-x-2">
        <input
          id="t"
          type="radio"
          className="h-4 w-4 border-gray-300 text-indigo-600 focus:ring-indigo-600"
          value="true"
          name="options"
          checked={value.typ === "bool" && value.value === true}
          onChange={handleOptionChange}
        />
        <label htmlFor="t" className="block">
          true
        </label>
      </div>
      <div className="flex items-center space-x-2">
        <input
          id="f"
          type="radio"
          className="h-4 w-4 border-gray-300 text-indigo-600 focus:ring-indigo-600"
          value="false"
          name="options"
          checked={value.typ === "bool" && value.value === false}
          onChange={handleOptionChange}
        />
        <label htmlFor="f" className="block">
          false
        </label>
      </div>
    </fieldset>
  );
}

interface TextColumnProps extends ColumnProps {
  line: number;
}

function ColumnText(props: TextColumnProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const columnType = props.columnType;
  const { name } = props.columnType;
  const value = rowEditorColumnValue(state, columnType);

  const placeholder = () => {
    switch (value.typ) {
      case "Default":
        return displayDefaultValue(columnType.defaultValue);
      default:
        return "";
    }
  };

  const displayValue = () => {
    switch (value.typ) {
      case "Not Defined":
      case "NULL":
      case "Default":
        return "";
      default:
        const v = value.value as string;
        return v;
    }
  };

  return (
    <textarea
      rows={props.line}
      name="comment"
      id="comment"
      className="block w-full rounded-md border-0 p-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
      placeholder={placeholder()}
      value={displayValue()}
      onChange={(e) => {
        dispatch({
          type: "SetColumnAction",
          columnName: name,
          columnAction: {
            typ: "SetRegular",
            value: { typ: "varchar(255)", value: e.target.value },
          },
        });
      }}
    />
  );
}

interface RowEditorProps {
  open: boolean;
  envId: string;
  tableName: string;
  beforeSave: () => void;
  afterSave: () => void;
}

function ColumnHeader(props: ColumnProps) {
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();
  const column = props.columnType;
  const currentValue = rowEditorColumnValue(state, column);
  const inputName = `nullDefaultOptions-${column.name}`;

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    const v = event.target.value;
    if (v === "isNull") {
      dispatch({
        type: "SetColumnAction",
        columnName: column.name,
        columnAction: { typ: "SetNull" },
      });
    } else if (v === "isDefault") {
      dispatch({
        type: "SetColumnAction",
        columnName: column.name,
        columnAction: { typ: "SetDefault" },
      });
    }
  };

  return (
    <div className="mr-auto flex justify-stretch">
      <label
        htmlFor="comment"
        className="block w-28 text-sm font-medium leading-6 text-gray-900"
      >
        {column.name}
      </label>
      <div className="ml-2 w-20 rounded-lg bg-blue-50 bg-gray-200 p-1 px-2 text-center text-xs">
        {column.fieldType}
      </div>

      {!isSystemField(column.name) && (
        <fieldset className="flex space-x-6 pl-6">
          {column.isNullable && (
            <div className="flex items-center space-x-2">
              <label htmlFor="isNull" className="block">
                Null
              </label>
              <input
                type="radio"
                name={inputName}
                value="isNull"
                checked={currentValue.typ === "NULL"}
                onChange={handleChange}
              />
            </div>
          )}

          {column.defaultValue.typ !== "Not Defined" &&
            column.defaultValue.typ !== "NULL" && (
              <div className="flex items-center space-x-2">
                <label htmlFor="isNull" className="block">
                  Default
                </label>
                <input
                  type="radio"
                  name={inputName}
                  value="isDefault"
                  checked={currentValue.typ === "Default"}
                  onChange={handleChange}
                />
              </div>
            )}
        </fieldset>
      )}
    </div>
  );
}

function ColumnContent(props: ColumnProps) {
  const column = props.columnType;
  if (isSystemField(column.name)) {
    return (
      <div className="ring-inset-0 mt-2 rounded p-2 text-sm text-gray-400 ring-1 ring-gray-300 focus:outline-none">
        Auto generated
      </div>
    );
  }

  switch (column.fieldType) {
    case "Not Defined":
      throw new Error("Not Defined");
    case "int64 Auto Increment":
      return (
        <div className="ring-inset-0 mt-2 rounded p-2 text-sm text-gray-400 ring-1 ring-gray-300 focus:outline-none">
          Auto generated
        </div>
      );
    case "int64":
    case "float64":
      return <ColumnNumber columnType={column}></ColumnNumber>;
    case "bool":
      return <ColumnBool columnType={column}></ColumnBool>;
    case "varchar(255)":
      return <ColumnText columnType={column} line={1}></ColumnText>;
    case "text":
      return <ColumnText columnType={column} line={2}></ColumnText>;
    case "datetime":
      return <ColumnDatetime columnType={column}></ColumnDatetime>;
  }
}

function ColumnContainer(props: ColumnProps) {
  const column = props.columnType;
  return (
    <div
      className="mt-6 rounded border bg-gray-50 p-3 shadow"
      key={column.name}
    >
      <ColumnHeader columnType={column}></ColumnHeader>
      <div className="mt-2">
        <ColumnContent columnType={column} />
      </div>
    </div>
  );
}

export default function RowEditor(props: RowEditorProps) {
  const { open } = props;
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const tableDef = state.schema[props.tableName]!;

  const handleCancel = () => {
    if (Object.keys(state.rowEditorColumnActions).length > 0) {
      setShowCancelConfirm(true);
    } else {
      dispatch({ type: "DeleteRowEditor" });
    }
  };

  const handleSave = async () => {
    switch (state.rowEditorMode) {
      case "Create":
        props.beforeSave();
        const insertR = getInsertRow(state);
        const apiR = RowDatumToApiRow(insertR);
        await insertRow(props.envId, props.tableName, apiR);
        props.afterSave();
        break;
      case "Update":
        props.beforeSave();
        const datum = state.rowEditorOriginalRow["id"]! as {
          typ: "int64AutoIncrement";
          value: number;
        };
        const id = datum.value;
        const updateR = getUpdateRow(state);
        const updateApiR = RowDatumToApiRow(updateR);
        await updateRow(props.envId, props.tableName, id, updateApiR);
        props.afterSave();
        break;
      case "None":
        throw new Error("RowEditor is not initialized");
    }
  };

  return (
    <>
      <Transition.Root show={open} as={Fragment}>
        <Dialog as="div" className="relative z-10" onClose={handleCancel}>
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
              <div className="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10">
                <Transition.Child
                  as={Fragment}
                  enter="transform transition ease-in-out duration-250"
                  enterFrom="translate-x-full"
                  enterTo="translate-x-0"
                  leave="transform transition ease-in-out duration-250"
                  leaveFrom="translate-x-0"
                  leaveTo="translate-x-full"
                >
                  <Dialog.Panel className="pointer-events-auto relative w-screen max-w-4xl">
                    <Transition.Child
                      as={Fragment}
                      enter="ease-in-out duration-250"
                      enterFrom="opacity-0"
                      enterTo="opacity-100"
                      leave="ease-in-out duration-250"
                      leaveFrom="opacity-100"
                      leaveTo="opacity-0"
                    >
                      <div className="absolute left-0 top-0 -ml-8 flex pr-2 pt-4 sm:-ml-10 sm:pr-4">
                        <button
                          type="button"
                          className="relative rounded-md text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-white"
                          onClick={handleCancel}
                        >
                          <span className="absolute -inset-2.5" />
                          <span className="sr-only">Close panel</span>
                          <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                        </button>
                      </div>
                    </Transition.Child>
                    <div className="flex h-full flex-col overflow-y-scroll bg-white py-6 shadow-xl">
                      <div className="px-4 sm:px-6">
                        <Dialog.Title className="text-base font-semibold leading-6 text-gray-900">
                          {state.rowEditorMode === "Create" && "New record"}
                          {state.rowEditorMode === "Update" &&
                            "Update an exising row"}
                        </Dialog.Title>
                      </div>
                      <div className="relative mt-6 flex-1 divide-y divide-gray-200 px-4 sm:px-6">
                        {tableDef.columns.map((column) => {
                          return (
                            <ColumnContainer
                              columnType={column}
                              key={column.name}
                            ></ColumnContainer>
                          );
                        })}
                      </div>
                      <div className="mr-6 mt-6 flex items-center justify-end gap-x-6">
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
                          className="w-36 rounded-md bg-gray-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-gray-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gray-600"
                          onClick={handleSave}
                        >
                          Save
                        </button>
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
          dispatch({ type: "DeleteRowEditor" });
          setShowCancelConfirm(false);
        }}
        onNo={() => {
          setShowCancelConfirm(false);
        }}
      ></DangerActionConfirm>
    </>
  );
}
