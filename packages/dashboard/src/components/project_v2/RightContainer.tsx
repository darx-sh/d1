import { useProjectState } from "~/components/project_v2/ProjectContext";

export default function RightContainer() {
  const projectState = useProjectState()!;
  const tabIdx = projectState.curOpenTabIdx;
  if (tabIdx === null) {
    return null;
  }

  console.log("tabs", projectState.tabs);
  console.log("curOpenTabIdx", projectState.curOpenTabIdx);
  const curOpenTab = projectState.tabs[tabIdx]!;
  if (curOpenTab.type !== "JsEditor") {
    return null;
  }

  const curCode = projectState.directory.codes[curOpenTab.codeIdx]!;
  const httpRoutes = projectState.directory.httpRoutes.filter((route) => {
    return route.jsEntryPoint === curCode.fsPath;
  });

  if (httpRoutes.length === 0) {
    return null;
  }

  return (
    <div className="mt-10">
      <ul role="list" className="divide-y divide-gray-100">
        {httpRoutes.map((route) => (
          <li
            key={route.httpPath}
            className="flex items-center justify-between gap-x-6 py-5"
          >
            <p className="ml-14 text-sm font-semibold leading-6 text-gray-900">
              {route.httpPath}
            </p>
            <a
              href="#"
              className="mr-14 rounded-full bg-white px-2.5 py-1 text-xs font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
            >
              invoke
            </a>
          </li>
        ))}
      </ul>
    </div>
  );
}
