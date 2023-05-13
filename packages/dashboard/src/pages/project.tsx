import { useState } from "react";
import {
  CircleStackIcon,
  CloudIcon,
  CodeBracketIcon,
  CodeBracketSquareIcon,
  ArrowPathIcon,
  ArrowTrendingUpIcon,
  Bars4Icon,
  WrenchIcon,
  UserGroupIcon,
} from "@heroicons/react/24/solid";

import Database from "~/components/project/database";
import Functions from "~/components/project/functions";
import Triggers from "~/components/project/triggers";
import Reports from "~/components/project/reports";
import Logs from "~/components/project/logs";
import Settings from "~/components/project/settings";
import Users from "~/components/project/users";

const navigation = [
  {
    name: "Database",
    icon: CircleStackIcon,
  },
  {
    name: "Functions",
    icon: CodeBracketSquareIcon,
  },
  {
    name: "Users",
    icon: UserGroupIcon,
  },
  {
    name: "Logs",
    icon: Bars4Icon,
  },
  {
    name: "Settings",
    icon: WrenchIcon,
  },
];

function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

export default function Example() {
  const [curIndex, setCurIndex] = useState(0);

  function handleNav(index: number) {
    setCurIndex(index);
  }

  return (
    <>
      <div>
        {/* Static sidebar for desktop */}
        <div className="lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-52 lg:flex-col">
          {/* Sidebar component, swap this element with another sidebar if you like */}
          <div className="flex grow flex-col gap-y-5 overflow-y-auto border-r border-gray-200 bg-white px-6">
            <div className="flex h-16 shrink-0 items-center">
              <img
                className="h-8 w-auto"
                src="https://tailwindui.com/img/logos/mark.svg?color=indigo&shade=600"
                alt="Your Company"
              />
            </div>
            <nav className="flex flex-1 flex-col">
              <ul role="list" className="flex flex-1 flex-col gap-y-7">
                <li>
                  <ul role="list" className="-mx-2 space-y-5">
                    {navigation.map((item, index) => (
                      <li
                        key={item.name}
                        onClick={() => {
                          handleNav(index);
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
                          <item.icon
                            className={classNames(
                              index === curIndex
                                ? "text-indigo-600"
                                : "text-gray-400 group-hover:text-indigo-600",
                              "h-6 w-6 shrink-0"
                            )}
                            aria-hidden="true"
                          />
                          {item.name}
                        </a>
                      </li>
                    ))}
                  </ul>
                </li>
                <li className="-mx-6 mt-auto">
                  <a
                    href="#"
                    className="flex items-center gap-x-4 px-6 py-3 text-sm font-semibold leading-6 text-gray-900 hover:bg-gray-50"
                  >
                    <img
                      className="h-8 w-8 rounded-full bg-gray-50"
                      src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                      alt=""
                    />
                    <span className="sr-only">Your profile</span>
                    <span aria-hidden="true">Tom Cook</span>
                  </a>
                </li>
              </ul>
            </nav>
          </div>
        </div>
        <main className="lg:pl-52">
          <div className="px-4 py-10 ">
            {curIndex === 0 && <Database />}
            {curIndex === 1 && <Functions />}
            {curIndex === 2 && <Users />}
            {curIndex === 3 && <Reports />}
            {curIndex === 4 && <Logs />}
            {curIndex === 5 && <Settings />}
          </div>
        </main>
      </div>
    </>
  );
}
