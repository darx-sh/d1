import {
  HttpRoute,
  useProjectState,
} from "~/components/project_v2/ProjectContext";
import { useState } from "react";
import InvokeModal from "~/components/project_v2/InvokeModal";

export default function RightContainer() {
  const projectState = useProjectState()!;
  const [invokingHttpRoute, setInvokingHttpRoute] = useState<HttpRoute | null>(
    null
  );

  const tabIdx = projectState.curOpenTabIdx;
  if (tabIdx === null) {
    return null;
  }

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

  const handleInvoke = (route: HttpRoute) => {
    setInvokingHttpRoute(route);
  };

  const handleCloseInvokeModal = () => {
    setInvokingHttpRoute(null);
  };

  return (
    <>
      {invokingHttpRoute !== null && (
        <InvokeModal
          httpRoute={invokingHttpRoute}
          onClose={handleCloseInvokeModal}
        ></InvokeModal>
      )}
      <div className="mt-10">
        <ul role="list" className="divide-y divide-gray-100">
          {httpRoutes.map((route) => (
            <li
              key={route.httpPath}
              className="flex items-center justify-between gap-x-6 py-5"
            >
              <p className="ml-14 text-sm font-semibold leading-6 text-gray-900">
                {"/" + route.httpPath}
              </p>
              <a
                href="#"
                className="mr-14 rounded-full bg-white px-2.5 py-1 text-xs font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-100"
                onClick={() => handleInvoke(route)}
              >
                invoke
              </a>
            </li>
          ))}
        </ul>
      </div>
    </>
  );
}
