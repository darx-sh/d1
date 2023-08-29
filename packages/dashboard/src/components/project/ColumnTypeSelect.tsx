import { Fragment, useState } from "react";
import { Listbox, Transition } from "@headlessui/react";
import { CheckIcon, ChevronDownIcon } from "@heroicons/react/20/solid";
import {
  FieldType,
  getAllColumnTypes,
} from "~/components/project/DatabaseContext";

// const columnOptions = [
//   {
//     columnType: "BIGINT",
//     description: "64 bit (8 bytes) integer",
//     current: true,
//   },
//   { columnType: "TEXT", description: "character strings", current: false },
// ];

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(" ");
}

interface ColumnTypeSelectProps {
  columnType: FieldType | null;
}

export default function ColumnTypeSelect(props: ColumnTypeSelectProps) {
  const allColumnTypes = getAllColumnTypes();
  const [selected, setSelected] = useState(props.columnType);

  return (
    <Listbox value={selected} onChange={setSelected}>
      {({ open }) => (
        <>
          <div className="relative rounded-md border shadow-md">
            <div className="flex divide-x divide-indigo-700">
              <div className="flex-1 gap-x-1.5 px-3 py-1.5">
                <p className="text-sm text-gray-400">{selected}</p>
              </div>
              <Listbox.Button className="w-8 bg-gray-400 p-2 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2 focus:ring-offset-gray-50">
                <ChevronDownIcon
                  className="h-3 w-3 text-white"
                  aria-hidden="true"
                />
              </Listbox.Button>
            </div>

            <Transition
              show={open}
              as={Fragment}
              leave="transition ease-in duration-100"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <Listbox.Options className="absolute left-0 z-10 mt-2 w-36 origin-top-right divide-y divide-gray-200 overflow-hidden rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
                {allColumnTypes.map((c) => (
                  <Listbox.Option
                    key={c}
                    className={({ active }) =>
                      classNames(
                        active ? "bg-gray-400 text-white" : "text-gray-900",
                        "cursor-default select-none p-4 text-sm"
                      )
                    }
                    value={c}
                  >
                    {({ selected, active }) => (
                      <div className="flex flex-col">
                        <div className="flex justify-between">
                          <p
                            className={
                              selected ? "font-semibold" : "font-normal"
                            }
                          >
                            {c}
                          </p>
                          {selected ? (
                            <span
                              className={
                                active ? "text-white" : "text-indigo-600"
                              }
                            >
                              <CheckIcon
                                className="h-5 w-5"
                                aria-hidden="true"
                              />
                            </span>
                          ) : null}
                        </div>
                        {/*<p*/}
                        {/*  className={classNames(*/}
                        {/*    active ? "text-indigo-200" : "text-gray-500",*/}
                        {/*    "mt-2"*/}
                        {/*  )}*/}
                        {/*>*/}
                        {/*  {option.description}*/}
                        {/*</p>*/}
                      </div>
                    )}
                  </Listbox.Option>
                ))}
              </Listbox.Options>
            </Transition>
          </div>
        </>
      )}
    </Listbox>
  );
}
