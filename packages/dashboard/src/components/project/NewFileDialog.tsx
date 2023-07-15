import React, { Fragment, useState, useRef } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { CheckIcon } from "@heroicons/react/24/outline";

type NewFileDialogProps = {
  closeDialog: () => void;
  handleNewFileName: (name: string) => void;
};

export default function NewFileDialog(props: NewFileDialogProps) {
  const [_, setIsOpen] = useState(true);
  const inputRef = useRef<HTMLInputElement>(null);
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Enter") {
      console.log("enter key");

      let newFileName = inputRef.current?.value;
      if (!newFileName) {
        newFileName = "";
      }
      props.closeDialog();
      props.handleNewFileName(newFileName);
    }
  };
  return (
    <Transition.Root show={true} as={Fragment}>
      <Dialog as="div" className="relative z-10" onClose={props.closeDialog}>
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" />
        </Transition.Child>

        <div className="fixed inset-0 z-10 overflow-y-auto">
          <div className="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
              enterTo="opacity-100 translate-y-0 sm:scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 translate-y-0 sm:scale-100"
              leaveTo="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
            >
              <Dialog.Panel className="relative transform overflow-hidden rounded-lg bg-white px-4 pb-4 pt-5 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-sm sm:p-6">
                <label
                  htmlFor="fileName"
                  className="block text-sm font-medium leading-6 text-gray-900"
                >
                  File name
                </label>
                <div className="mt-2">
                  <input
                    ref={inputRef}
                    onKeyDown={handleKeyDown}
                    type="text"
                    name="file name"
                    id="file name"
                    className="block w-full rounded-md border-0 px-1 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
                    placeholder="api.js"
                    aria-describedby="file-name-description"
                  />
                </div>
                <p
                  className="mt-2 text-sm text-gray-500"
                  id="file-name-description"
                >
                  Only .js and .ts files are supported.
                </p>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition.Root>
  );
}
