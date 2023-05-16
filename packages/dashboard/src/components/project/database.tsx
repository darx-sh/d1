import { useState } from "react";
import Data from "~/components/project/database/data";
import Schema from "~/components/project/database/schema";

const navigation = [
  { id: "Data", href: "#", current: false },
  { id: "Schema", href: "#", current: false },
];

function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

export default function Database() {
  const [curIndex, setCurIndex] = useState(0);
  return (
    <div className="mx-2 my-4 flex rounded-lg border bg-slate-50 px-4 py-10">
      <div className="flex-none  p-4">
        <ul role="list" className="-mx-2 space-y-2">
          {navigation.map((item, index) => (
            <li
              key={item.id}
              onClick={() => {
                setCurIndex(index);
              }}
              className={classNames(
                index === curIndex
                  ? "bg-slate-200 text-indigo-600"
                  : "text-gray-700 hover:bg-gray-50 hover:text-indigo-600",
                "group flex gap-x-3 rounded-md p-2 text-sm font-semibold leading-6"
              )}
            >
              {item.id}
            </li>
          ))}
        </ul>
      </div>
      <div className="flex-none p-4">
        {curIndex === 0 && <Data />}
        {curIndex === 1 && <Schema />}
      </div>
    </div>
  );
}
