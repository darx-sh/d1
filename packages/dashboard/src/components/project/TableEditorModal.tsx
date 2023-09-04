import { useState, useEffect, Fragment, useRef } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon, ExclamationCircleIcon } from "@heroicons/react/24/outline";
import ColumnsEditor from "~/components/project/ColumnsEditor";
import {
  DxFieldType,
  DxColumnType,
  ColumnError,
  TableDef,
  TableDefError,
  useDatabaseDispatch,
  useDatabaseState,
  defaultValueToJSON,
} from "~/components/project/DatabaseContext";
import { useProjectState } from "~/components/project/ProjectContext";
import { invoke, type CreateTableReq } from "~/utils";

type EditTableReq = AddColumnReq | DropColumnReq | RenameColumnReq;

interface DropTableReq {
  dropTable: {
    tableName: string;
  };
}

interface AddColumnReq {
  addColumn: {
    tableName: string;
    column: DxColumnType;
  };
}

interface DropColumnReq {
  dropColumn: {
    tableName: string;
    columnName: string;
  };
}

interface RenameColumnReq {
  renameColumn: {
    tableName: string;
    oldColumnName: string;
    newColumnName: string;
  };
}

type TableEditorProps = {
  onClose: () => void;
  open: boolean;
  onCreateTable: () => void;
  onEditTable: () => void;
};

export default function TableEditorModal(props: TableEditorProps) {
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();
  const { onClose, open } = props;
  const tableDef = state.draftTable;
  const tableDefError = state.draftTableError;
  const tableNameRef = useRef<HTMLInputElement>(null);

  const handleSave = () => {
    if (state.isDraftFromTemplate) {
      props.onCreateTable();
    } else {
      props.onEditTable();
    }
  };

  return (
    <Transition.Root show={open} as={Fragment}>
      <Dialog
        as="div"
        className="relative z-10"
        onClose={() => {
          onClose();
        }}
        initialFocus={state.isDraftFromTemplate ? tableNameRef : undefined}
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
                            onClick={() => {
                              onClose();
                            }}
                          >
                            <span className="absolute -inset-2.5" />
                            <span className="sr-only">Close panel</span>
                            <XMarkIcon className="h-6 w-6" aria-hidden="true" />
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
                                        ? "block rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
                                        : "block rounded-md border-0 py-1.5 text-red-900 shadow-sm ring-1 ring-inset ring-red-300 placeholder:text-red-400 focus:ring-2 focus:ring-inset focus:ring-red-600"
                                    }
                                    onChange={(event) => {
                                      const v = event.target.value;
                                      dispatch({
                                        type: "SetTableName",
                                        tableName: v,
                                      });
                                    }}
                                  />
                                  {tableDefError.nameError === null ? null : (
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
  );
}
