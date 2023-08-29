import { useState } from "react";
import { useDatabaseState, useDatabaseDispatch } from "./DatabaseContext";
import TableEditorModal from "~/components/project/TableEditorModal";

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(" ");
}

export default function TableList() {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const [isCreateTable, setIsCreateTable] = useState(false);

  const navigation = Object.keys(state.schema).map((tableName: string) => {
    return { name: tableName, href: "#", current: false };
  });

  return (
    <>
      <TableEditorModal
        open={isCreateTable}
        onClose={() => {
          dispatch({ type: "DeleteScratchTable" });
          setIsCreateTable(false);
        }}
        prepareDDL={false}
      ></TableEditorModal>

      <nav className="flex flex-col p-2" aria-label="Tables">
        <button
          type="button"
          className="mb-2 block rounded bg-gray-400 px-2 py-2 text-left text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
          onClick={() => {
            setIsCreateTable(true);
          }}
        >
          Create Table
        </button>
        <ul role="list" className="space-y-1">
          {navigation.map((item) => (
            <li key={item.name}>
              <a
                href={item.href}
                className={classNames(
                  item.current
                    ? "bg-gray-50 text-indigo-600"
                    : "text-gray-700 hover:bg-gray-50 hover:text-indigo-600",
                  "group flex gap-x-3 rounded-md p-2 pl-3 text-sm font-semibold leading-6"
                )}
              >
                {item.name}
              </a>
            </li>
          ))}
        </ul>
      </nav>
    </>
  );
}
