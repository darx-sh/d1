import { Fragment, useState } from "react";
import { Dialog, Menu, Transition } from "@headlessui/react";

import {
  EllipsisVerticalIcon,
  EllipsisHorizontalIcon,
} from "@heroicons/react/20/solid";
import className from "classnames";

export default function ColumnActionDropDown() {
  const [showRenameColumn, setShowRenameColumn] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const renameColumn = () => {
    return (
      <Dialog
        as="div"
        open={showRenameColumn}
        onClose={() => setShowRenameColumn(false)}
        className="z-10"
      >
        <div style={{ position: "fixed", top: position.y, left: position.x }}>
          <Dialog.Panel>
            <div className="text-2xl">Some stuff</div>
          </Dialog.Panel>
        </div>
      </Dialog>
    );
  };

  return (
    <>
      {renameColumn()}
      <Menu as="div" className="relative mr-6 inline-block text-left">
        <div>
          <Menu.Button className="flex items-center rounded-full bg-gray-100 text-gray-400 hover:text-gray-600 focus:outline-none focus:ring-offset-2 focus:ring-offset-gray-100">
            <EllipsisHorizontalIcon className="h-6 w-6" aria-hidden="true" />
          </Menu.Button>
        </div>

        <Transition
          as={Fragment}
          enter="transition ease-out duration-100"
          enterFrom="transform opacity-0 scale-95"
          enterTo="transform opacity-100 scale-100"
          leave="transition ease-in duration-75"
          leaveFrom="transform opacity-100 scale-100"
          leaveTo="transform opacity-0 scale-95"
        >
          <Menu.Items className="absolute right-0 z-10 mt-2 w-40 origin-top-right rounded-md bg-gray-50 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
            <div className="py-1">
              <Menu.Item>
                {({ active }) => (
                  <a
                    href="#"
                    className={className(
                      active ? "bg-gray-100 text-gray-900" : "text-gray-700",
                      "block px-4 py-2 text-sm"
                    )}
                    onClick={(e) => {
                      setPosition({ x: e.clientX, y: e.clientY });
                      setShowRenameColumn(true);
                    }}
                  >
                    Rename Column
                  </a>
                )}
              </Menu.Item>
              <Menu.Item>
                {({ active }) => (
                  <a
                    href="#"
                    className={className(
                      active ? "bg-gray-100 text-gray-900" : "text-gray-700",
                      "block px-4 py-2 text-sm"
                    )}
                  >
                    Delete Column
                  </a>
                )}
              </Menu.Item>
            </div>
          </Menu.Items>
        </Transition>
      </Menu>
    </>
  );
}
