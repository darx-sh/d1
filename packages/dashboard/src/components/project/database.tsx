import { useState } from "react";

const navigation = [
  { name: "Query", href: "#", current: false },
  { name: "Schema Editor", href: "#", current: false },
];

function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

export default function Database() {
  const [curIndex, setCurIndex] = useState(0);
  return (
    <div>
      <div className="border-r border-gray-200 bg-white">
        <ul role="list" className="-mx-2 space-y-5">
          {navigation.map((item, index) => (
            <li key={item.name}>
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
    </div>
  );
}
