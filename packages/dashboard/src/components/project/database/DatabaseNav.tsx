import { useDatabaseState, useDatabaseDispatch, Row } from "./DatabaseContext";
import classNames from "classnames";

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
      <nav className="mb-6 mt-4 flex flex-col rounded p-2">
        <ul className="space-y-1">
          <li
            className=""
            onClick={() => {
              dispatch({ type: "SetNav", nav: { typ: "Schema" } });
            }}
          >
            Schema
          </li>
        </ul>
      </nav>
      <div className="">
        <p className="p-2">Tables</p>
        <nav className="flex flex-col p-2" aria-label="Tables">
          <ul role="list" className="space-y-1">
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
                  {item.name}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </div>
    </>
  );
}
