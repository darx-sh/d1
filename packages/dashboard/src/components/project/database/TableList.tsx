import { useState } from "react";
import { useDatabaseState, useDatabaseDispatch, Row } from "./DatabaseContext";
import TableEditorModal from "~/components/project/database/TableEditorModal";
import { env } from "~/env.mjs";
import axios from "axios";
import { useProjectState } from "~/components/project/ProjectContext";
import Spinner from "~/components/project/Spinner";

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(" ");
}

interface TableListProps {
  onSelectTable: (tableName: string) => void;
}

export default function TableList(props: TableListProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const projectState = useProjectState();
  const curTableName = state.curWorkingTable?.tableName;

  const navigation = Object.keys(state.schema).map((tableName: string) => {
    return { name: tableName, href: "#", current: curTableName === tableName };
  });

  return (
    <>
      <nav className="flex flex-col p-2" aria-label="Tables">
        <ul role="list" className="space-y-1">
          {navigation.map((item) => (
            <li
              key={item.name}
              onClick={() => {
                props.onSelectTable(item.name);
              }}
            >
              <a
                href={item.href}
                className={classNames(
                  item.current
                    ? "bg-gray-200 text-indigo-600"
                    : "text-gray-700 hover:bg-gray-50 hover:text-indigo-600",
                  "group flex gap-x-3 rounded-sm p-2 pl-3 text-sm font-semibold leading-6"
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
