import {
  useProjectState,
  useProjectDispatch,
} from "~/components/project_v2/ProjectContext";

import { classNames } from "~/utils";

export default function MiddleToolTabs() {
  const projectState = useProjectState()!;
  const projectDispatch = useProjectDispatch()!;
  const tabs = projectState.tabs.map((tab, idx) => {
    switch (tab.type) {
      case "JsEditor": {
        const codeIdx = tab.codeIdx;
        const fileName = projectState.directory.codes[codeIdx]!.fsPath;
        if (idx === projectState.curOpenTabIdx) {
          return {
            name: fileName,
            href: "#",
            current: true,
          };
        } else {
          return {
            name: fileName,
            href: "#",
            current: false,
          };
        }
      }
      case "Database": {
        if (idx === projectState.curOpenTabIdx) {
          return {
            name: "Database",
            href: "#",
            current: true,
          };
        } else {
          return {
            name: "Database",
            href: "#",
            current: false,
          };
        }
      }
    }
  });

  return (
    <nav className="flex divide-x-2" aria-label="Tabs">
      {tabs.map((tab, tabIdx) => (
        <a
          key={tab.name}
          href={tab.href}
          className={classNames(
            tab.current
              ? "border-b-2 border-b-blue-300 bg-white text-gray-900 shadow"
              : "text-gray-500 hover:text-gray-700",
            "px-4 py-2 text-center text-xs focus:z-10"
          )}
          aria-current={tab.current ? "page" : undefined}
          onClick={() => {
            projectDispatch({
              type: "SelectTab",
              tabIdx,
            });
          }}
        >
          {tab.name}
        </a>
      ))}
    </nav>
  );
}
