import {
  CircleStackIcon,
  ArrowTrendingUpIcon,
  Bars3Icon,
  LockClosedIcon,
} from "@heroicons/react/24/outline";
import { useProjectDispatch, useProjectState } from "./ProjectContext";
import className from "classnames";

export default function LeftTools() {
  const dispatch = useProjectDispatch();
  const state = useProjectState();

  const curTab = () => {
    if (state.curOpenTabIdx === null) {
      return null;
    }
    return state.tabs[state.curOpenTabIdx];
  };

  const toolClass = (typ: string) => {
    const commonClass =
      "flex h-16 w-16 flex flex-col items-center border rounded p-4 hover:bg-gray-100 cursor-pointer";
    const highlightClass = "outline outline-blue-300 shadow bg-gray-100";
    if (curTab()?.type == typ) {
      return className(commonClass, highlightClass);
    } else {
      return className(commonClass, "");
    }
  };

  return (
    <div className="mt-6 flex flex-wrap justify-evenly gap-x-2 gap-y-2">
      <div
        className={toolClass("Database")}
        onClick={() => dispatch({ type: "OpenDatabase" })}
      >
        <CircleStackIcon className="h-5 w-5" />
        <div className="text-sm">Database</div>
      </div>
      <div className={toolClass("Reports")}>
        <ArrowTrendingUpIcon className="h-5 w-5" />
        <div className="text-sm">Reports</div>
      </div>
      <div className={toolClass("Logs")}>
        <Bars3Icon className="h-5 w-5" />
        <div className="text-sm">Logs</div>
      </div>
      <div className={toolClass("Secrets")}>
        <LockClosedIcon className="h-5 w-5" />
        <div className="text-sm">Secrets</div>
      </div>
    </div>
  );
}
