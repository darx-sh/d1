import { Menu } from "@headlessui/react";
import React, { useEffect } from "react";

import {
  DocumentPlusIcon,
  DocumentMinusIcon,
  FolderPlusIcon,
  PencilIcon,
} from "@heroicons/react/24/solid";

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(" ");
}

type FileMenuProps = {
  menuPosition: { x: number; y: number };
  hideMenu: () => void;
  handleNewFile: () => void;
};
export default function FileMenu(props: FileMenuProps) {
  useEffect(() => {
    const handleClick = () => {
      props.hideMenu();
    };

    window.addEventListener("click", handleClick);

    return () => {
      window.removeEventListener("click", handleClick);
    };
  }, []);

  const handleNewFileClick = () => {
    props.handleNewFile();
  };

  const handleDeleteClick = () => {
    console.log("delete clicked");
  };

  return (
    <div
      style={{
        position: "fixed",
        top: props.menuPosition.y,
        left: props.menuPosition.x - 128,
      }}
    >
      <Menu>
        <Menu.Items
          static
          className="z-10 mt-2 w-32 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none"
        >
          <div className="py-1">
            <Menu.Item>
              {({ active }) => (
                <a
                  href="#"
                  className={classNames(
                    active ? "bg-gray-100 text-gray-900" : "text-gray-700",
                    "group flex items-center px-4 py-2 text-sm"
                  )}
                  onClick={handleNewFileClick}
                >
                  <DocumentPlusIcon
                    className="mr-3 h-5 w-5 text-gray-400 group-hover:text-gray-500"
                    aria-hidden="true"
                  />
                  New File
                </a>
              )}
            </Menu.Item>
          </div>
          <div className="py-1">
            <Menu.Item>
              {({ active }) => (
                <a
                  href="#"
                  className={classNames(
                    active ? "bg-gray-100 text-gray-900" : "text-gray-700",
                    "group flex items-center px-4 py-2 text-sm"
                  )}
                  onClick={handleDeleteClick}
                >
                  <DocumentMinusIcon
                    className="mr-3 h-5 w-5 text-gray-400 group-hover:text-gray-500"
                    aria-hidden="true"
                  />
                  Delete
                </a>
              )}
            </Menu.Item>
          </div>
        </Menu.Items>
      </Menu>
    </div>
  );
}
