import { useState } from "react";
import { useRouter } from "next/router";
import {
  CircleStackIcon,
  CodeBracketSquareIcon,
  Bars4Icon,
  WrenchIcon,
} from "@heroicons/react/24/solid";

import Database from "~/components/project/database";
import Functions from "~/components/project/Functions";
import Logs from "~/components/project/logs";
import Settings from "~/components/project/settings";
import Users from "~/components/project/users";
import Plugins from "~/components/project/plugins";
import QuickNav from "~/components/project/QuickNav";

const navigation = [
  {
    id: "Database",
    icon: CircleStackIcon,
  },
  {
    id: "Code",
    icon: CodeBracketSquareIcon,
  },
  {
    id: "Logs",
    icon: Bars4Icon,
  },
  {
    id: "Settings",
    icon: WrenchIcon,
  },
];

function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

export default function ProjectDetail() {
  const [curIndex, setCurIndex] = useState(0);
  const router = useRouter();
  const projectId = router.query.id as string;
  const quickNav = [
    { name: "Home", href: "/" },
    { name: "Projects", href: "/projects" },
    { name: `${projectId}`, href: "#" },
  ];

  function handleNav(index: number) {
    setCurIndex(index);
  }

  const MainNav = () => {
    return (
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
              <ul role="list" className="-mx-2 space-y-4">
                {navigation.map((item, index) => (
                  <li
                    key={item.id}
                    onClick={() => {
                      handleNav(index);
                    }}
                  >
                    <a
                      className={classNames(
                        index === curIndex
                          ? "bg-gray-200 text-indigo-600"
                          : "text-gray-700 hover:bg-gray-200 hover:text-indigo-600",
                        "group flex gap-x-3 rounded p-2 text-sm font-semibold leading-6"
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
                      {item.id}
                    </a>
                  </li>
                ))}
              </ul>
            </li>
            <li className="-mx-6 mt-auto">
              <a
                href="./index.tsx#"
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
    );
  };

  return (
    <div className="h-screen bg-gray-100">
      <div className="fixed inset-y-0 z-50 flex w-48">
        <MainNav></MainNav>
      </div>
      <main className="fixed inset-y-0 ml-48 w-full">
        <QuickNav nav={quickNav} />
        {curIndex === 0 && <Database />}
        {curIndex === 1 && <Functions />}
        {curIndex === 2 && <Users />}
        {curIndex === 3 && <Plugins />}
        {curIndex === 4 && <Logs />}
        {curIndex === 5 && <Settings />}
      </main>
    </div>
  );
}
