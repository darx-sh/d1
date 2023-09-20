import {
  CircleStackIcon,
  ArrowTrendingUpIcon,
  Bars3Icon,
  LockClosedIcon,
  CodeBracketIcon,
} from "@heroicons/react/24/outline";
import { useProjectDispatch, useProjectState, NavType } from "./ProjectContext";
import className from "classnames";

export default function LeftNav() {
  const dispatch = useProjectDispatch();
  const state = useProjectState();

  const toolClass = (navType: NavType) => {
    const commonClass =
      "flex w-20 m-auto mb-2 flex flex-col items-center p-4 hover:bg-gray-100 cursor-pointer";
    const highlightClass = "outline outline-blue-300 shadow bg-gray-100";
    if (state.curNav === navType) {
      return className(commonClass, highlightClass);
    } else {
      return className(commonClass, "");
    }
  };

  return (
    <div className="mt-6 justify-evenly gap-x-2 gap-y-2 divide-gray-900">
      <div
        className={toolClass("database")}
        onClick={() => dispatch({ type: "OpenDatabase" })}
      >
        <CircleStackIcon className="h-5 w-5" />
        <div className="text-xs">Database</div>
      </div>
      <div
        className={toolClass("functions")}
        onClick={() => dispatch({ type: "OpenFunctions" })}
      >
        <CodeBracketIcon className="h-5 w-5" />
        <div className="text-xs">Functions</div>
      </div>

      <div className={toolClass("metrics")}>
        <ArrowTrendingUpIcon className="h-5 w-5" />
        <div className="text-xs">Metrics</div>
      </div>
      <div className={toolClass("logs")}>
        <Bars3Icon className="h-5 w-5" />
        <div className="text-xs">Logs</div>
      </div>
      <div className={toolClass("secrets")}>
        <LockClosedIcon className="h-5 w-5" />
        <div className="text-xs">Secrets</div>
      </div>
    </div>
  );
}
