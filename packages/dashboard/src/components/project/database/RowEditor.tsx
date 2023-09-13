import { ChangeEvent, Fragment, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import DangerActionConfirm from "~/components/project/database/DangerActionConfirm";
import {
  useDatabaseState,
  useDatabaseDispatch,
  rowChanged,
  DxColumnType,
  Row,
  isSystemField,
} from "~/components/project/database/DatabaseContext";
import { displayFieldType } from "~/utils/types";
import { insertRow } from "~/components/project/database/Api";
import { DateTimePicker } from "@mui/x-date-pickers/DateTimePicker";
import { Dayjs } from "dayjs";
import dayjs from "dayjs";

interface ColumnProps {
  columnType: DxColumnType;
}

function ColumnDatetime(props: ColumnProps) {
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();
  const value = state.draftRow[props.columnType.name] as string | undefined;
  let v: Dayjs | null = null;
  if (value !== undefined) {
    v = dayjs(state.draftRow[props.columnType.name] as string);
  }
  return (
    <DateTimePicker
      ampm={false}
      defaultValue={v}
      onChange={(value: Dayjs | null, _context) => {
        if (value !== null) {
          const dt = value.format("2021-06-30 13:03:47.123Z");
          dispatch({
            type: "SetColumnValue",
            columnName: props.columnType.name,
            value: dt,
          });
        } else {
          dispatch({
            type: "SetColumnNullMark",
            columnName: props.columnType.name,
            isNull: true,
          });
        }
      }}
    ></DateTimePicker>
  );
}

function ColumnNumber(props: ColumnProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const value = state.draftRow[props.columnType.name] as number | undefined;
  const { name } = props.columnType;
  return (
    <input
      type="number"
      className="ring-inset-0 mt-2 rounded p-2 text-sm text-gray-900 ring-1 ring-gray-300"
      onChange={(e) => {
        const value = e.target.value;
        if (value === "") {
          if (props.columnType.isNullable) {
            dispatch({
              type: "SetColumnNullMark",
              columnName: name,
              isNull: true,
            });
          }
        } else {
          dispatch({
            type: "SetColumnValue",
            columnName: name,
            value: value,
          });
        }
      }}
    >
      {value}
    </input>
  );
}

function ColumnBool(props: ColumnProps) {
  const dispatch = useDatabaseDispatch();
  const { name } = props.columnType;

  const handleOptionChange = (e: ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    if (v === "true") {
      dispatch({ type: "SetColumnValue", columnName: name, value: true });
    } else {
      dispatch({ type: "SetColumnValue", columnName: name, value: false });
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
  const { name } = props.columnType;
  const value = state.draftRow[name] as string | undefined;

  return (
    <textarea
      rows={props.line}
      name="comment"
      id="comment"
      className="block w-full rounded-md border-0 p-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
      value={value || ""}
      onChange={(e) => {
        dispatch({
          type: "SetColumnValue",
          columnName: name,
          value: e.target.value,
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

export default function RowEditor(props: RowEditorProps) {
  const { open } = props;
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const tableDef = state.schema[props.tableName]!;

  const handleCancel = () => {
    if (
      rowChanged(state.draftRowMode, state.draftOriginalRow, state.draftRow)
    ) {
      setShowCancelConfirm(true);
    } else {
      dispatch({ type: "DeleteRowEditor" });
    }
  };

  const handleSave = async () => {
    switch (state.draftRowMode) {
      case "Create":
        props.beforeSave();
        await insertRow(props.envId, props.tableName, state.draftRow);
        props.afterSave();
        break;
      case "Update":
        props.beforeSave();
        // todo
        props.afterSave();
        break;
      case "None":
        throw new Error("RowEditor is not initialized");
    }
  };

  const renderColumnHeader = (column: DxColumnType) => {
    return (
      <div className="mr-auto flex justify-stretch">
        <label
          htmlFor="comment"
          className="block w-28 text-sm font-medium leading-6 text-gray-900"
        >
          {column.name}
        </label>
        <div className="ml-2 w-20 rounded-lg bg-blue-50 bg-gray-200 p-1 px-2 text-center text-xs">
          {displayFieldType(column.fieldType)}
        </div>

        {column.isNullable && (
          <div className="flex h-6 items-center px-4">
            <label
              htmlFor="comment"
              className="w-18 mr-2 block text-sm font-normal leading-6 text-gray-900"
            >
              NULL
            </label>
            <input
              id="comments"
              aria-describedby="comments-description"
              name="comments"
              type="checkbox"
              checked={columnIsNull(column.name)}
              className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
              onChange={(e) => {
                const isNull = e.target.checked;
                dispatch({
                  type: "SetColumnNullMark",
                  columnName: column.name,
                  isNull,
                });
              }}
            />
          </div>
        )}
      </div>
    );
  };

  const columnIsNull = (columnName: string) => {
    let valueIsNull = false;
    if (state.draftRowMode === "Create" && state.draftRowNullMark[columnName] === true) {
      valueIsNull = true;
    }
    if (state.draftRowMode === "Update" && state.draftRow[columnName] === null) {
      valueIsNull = true;
    }
    return valueIsNull;
  }

  const renderColumn = (column: DxColumnType, row: Row) => {
    const renderColumnContent = () => {
      if (isSystemField(column.name)) {
        return (
          <div className="ring-inset-0 mt-2 rounded p-2 text-sm text-gray-400 ring-1 ring-gray-300 focus:outline-none">
            Auto generated
          </div>
        );
      }

      switch (column.fieldType) {
        case "int64Identity":
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
    };



    return (
      <div
        className="mt-6 rounded border bg-gray-50 p-3 shadow"
        key={column.name}
      >
        {renderColumnHeader(column)}
        <div className="mt-2">{!columnIsNull(column.name) && renderColumnContent()}</div>
      </div>
    );
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
                          {state.draftRowMode === "Create" && "New record"}
                          {state.draftRowMode === "Update" &&
                            "Update an exising row"}
                        </Dialog.Title>
                      </div>
                      <div className="relative mt-6 flex-1 divide-y divide-gray-200 px-4 sm:px-6">
                        {tableDef.columns.map((column) => {
                          return renderColumn(column, state.draftRow);
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
          dispatch({type: "DeleteDraftRow"});
          setShowCancelConfirm(false);
        }}
        onNo={() => {
          setShowCancelConfirm(false);
        }}
      ></DangerActionConfirm>
    </>
  );
}
