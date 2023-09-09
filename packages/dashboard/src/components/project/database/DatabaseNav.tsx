import { useDatabaseState, useDatabaseDispatch, Row } from "./DatabaseContext";
import classNames from "classnames";
import { ShareIcon, TableCellsIcon } from "@heroicons/react/20/solid";

export default function DatabaseNav() {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();

  let curTableName: string | null = null;
  if (state.curNav.typ === "Table") {
    curTableName = state.curNav.tableName;
  }

  const navigation = Object.keys(state.schema).map((tableName: string) => {
    return { name: tableName, href: "#", current: curTableName === tableName };
  });

  return (
    <>
      <div className="mx-2 mt-6 rounded bg-gray-200 px-2 py-3">
        <div
          className={classNames(
            state.curNav.typ === "Schema"
              ? "text-indigo-600"
              : "text-gray-700 hover:bg-gray-50 hover:text-indigo-600",
            "flex items-center justify-center py-3"
          )}
          onClick={() => {
            dispatch({ type: "SetNav", nav: { typ: "Schema" } });
          }}
        >
          <ShareIcon className="h-5 w-5" />
          <div className="px-2 text-sm font-semibold leading-6">Schema</div>
        </div>
      </div>

      <div className="mx-2 mt-6 rounded bg-gray-200 py-4">
        <div className="px-2 text-xs">Tables</div>
        <nav className="flex flex-col px-2" aria-label="Tables">
          <ul
            role="list"
            className="space-y-1 divide-y-2 border-b-2 border-t-2"
          >
            {navigation.map((item) => (
              <li
                key={item.name}
                onClick={() => {
                  dispatch({
                    type: "SetNav",
                    nav: { typ: "Table", tableName: item.name },
                  });
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
                  <div className="flex items-center">
                    <div>
                      <TableCellsIcon className="h-5 w-5"></TableCellsIcon>
                    </div>
                    <div className="px-2 text-sm">{item.name}</div>
                  </div>
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </div>
    </>
  );
}
