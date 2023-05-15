import { useState } from "react";
import Data from "~/components/project/database/data";
import Schema from "~/components/project/database/schema";

const navigation = [
  { name: "Data", href: "#", current: false },
  { name: "Schema", href: "#", current: false },
];

function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

export default function Database() {
  const [curIndex, setCurIndex] = useState(0);
  return (
    <div className="grid grid-cols-12 gap-2">
      <div className="border-r border-gray-200 bg-white">
        <ul role="list" className="-mx-2 space-y-2">
          {navigation.map((item, index) => (
            <li
              key={item.name}
              onClick={() => {
                setCurIndex(index);
              }}
            >
              <a
                className={classNames(
                  index === curIndex
                    ? "bg-gray-50 text-indigo-600"
                    : "text-gray-700 hover:bg-gray-50 hover:text-indigo-600",
                  "group flex gap-x-3 rounded-md p-2 text-sm font-semibold leading-6"
                )}
              >
                {item.name}
              </a>
            </li>
          ))}
        </ul>
      </div>
      <div className="col-span-11">
        {curIndex === 0 && <Data />}
        {curIndex === 1 && <Schema />}
      </div>
    </div>
  );
}
