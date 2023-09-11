import { Fragment, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import DangerActionConfirm from "~/components/project/database/DangerActionConfirm";
import {
  useDatabaseState,
  useDatabaseDispatch,
  rowChanged,
  DxColumnType,
  Row,
} from "~/components/project/database/DatabaseContext";
import { displayFieldType } from "~/utils/types";
import { DateTimePicker } from "@mui/x-date-pickers/DateTimePicker";

function ColumnDatetime() {
  return <DateTimePicker ampm={false}></DateTimePicker>;
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

  const handleClose = () => {
    if (rowChanged(state.draftOriginalRow, state.draftRow)) {
      setShowCancelConfirm(true);
    } else {
      dispatch({ type: "DeleteRowEditor" });
    }
  };

  const renderNull = () => {
    return <div>NULL</div>;
  };

  const renderColumnName = (column: DxColumnType) => {
    return (
      <div className="flex" key={column.name}>
        <label
          htmlFor="comment"
          className="block w-28 text-sm font-medium leading-6 text-gray-900"
        >
          {column.name}
        </label>
        <div className="ml-2 rounded-lg bg-blue-50 bg-gray-200 p-1 px-2 text-center text-xs">
          {displayFieldType(column.fieldType)}
        </div>
      </div>
    );
  };

  const renderNumber = (
    column: DxColumnType,
    value: number | null | undefined
  ) => {
    return (
      <div className="mt-6" key={column.name}>
        {renderColumnName(column)}
        <div>
          <div className="ring-inset-0 mt-2 rounded p-2 ring-1 ring-gray-200">
            {value === null ? (
              renderNull()
            ) : (
              <input type="number">{value}</input>
            )}
          </div>
        </div>
      </div>
    );
  };

  const renderIdentity = (column: DxColumnType) => {
    return (
      <div className="mt-6" key={column.name}>
        {renderColumnName(column)}

        <div className="ring-inset-0 mt-2 rounded p-2 text-sm text-gray-400 ring-1 ring-gray-200">
          Auto generated
        </div>
      </div>
    );
  };

  const renderBool = (
    column: DxColumnType,
    value: boolean | null | undefined
  ) => {
    return (
      <div className="mt-6" key={column.name}>
        {renderColumnName(column)}
        <div className="mt-2">
          {value === null ? renderNull() : <input type="number">{value}</input>}
        </div>
      </div>
    );
  };

  const renderVarChar = (
    column: DxColumnType,
    value: string | null | undefined
  ) => {
    return (
      <div className="mt-6" key={column.name}>
        {renderColumnName(column)}
        <div className="mt-2">
          {value === null ? (
            renderNull()
          ) : (
            <textarea
              rows={1}
              name="comment"
              id="comment"
              className="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
              defaultValue={value}
            />
          )}
        </div>
      </div>
    );
  };

  const renderText = (
    column: DxColumnType,
    value: string | null | undefined
  ) => {
    return (
      <div className="mt-6" key={column.name}>
        {renderColumnName(column)}
        <div className="mt-2">
          {value === null ? (
            renderNull()
          ) : (
            <textarea
              rows={3}
              className="block w-full rounded-md border-0 px-1.5 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gray-300 sm:text-sm sm:leading-6"
              defaultValue={value}
            />
          )}
        </div>
      </div>
    );
  };

  const renderDatetime = (
    column: DxColumnType,
    value: string | null | undefined
  ) => {
    return (
      <div className="mt-6" key={column.name}>
        {renderColumnName(column)}
        <div className="mt-2">
          {value === null ? renderNull() : <ColumnDatetime></ColumnDatetime>}
        </div>
      </div>
    );
  };

  const renderColumn = (column: DxColumnType, row: Row) => {
    const value = row[column.name];
    switch (column.fieldType) {
      case "int64Identity":
        return renderIdentity(column);
      case "int64":
      case "float64":
        return renderNumber(
          column,
          value! as unknown as number | null | undefined
        );
      case "bool":
        return renderBool(
          column,
          value! as unknown as boolean | null | undefined
        );
      case "varchar(255)":
        return renderVarChar(
          column,
          value! as unknown as string | null | undefined
        );
      case "text":
        return renderText(
          column,
          value! as unknown as string | null | undefined
        );
      case "datetime":
        return renderDatetime(
          column,
          value! as unknown as string | null | undefined
        );
    }
  };

  return (
    <>
      <Transition.Root show={open} as={Fragment}>
        <Dialog as="div" className="relative z-10" onClose={handleClose}>
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
                          onClick={handleClose}
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
                          {state.draftRowMod === "Create" && "New record"}
                          {state.draftRowMod === "Update" &&
                            "Update an exising row"}
                        </Dialog.Title>
                      </div>
                      <div className="relative mt-6 flex-1 px-4 sm:px-6">
                        {tableDef.columns.map((column) => {
                          return renderColumn(column, state.draftRow);
                        })}
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
        }}
        onNo={() => {
          setShowCancelConfirm(false);
        }}
      ></DangerActionConfirm>
    </>
  );
}
