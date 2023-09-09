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
    const defaultClass =
      "flex h-20 w-20 flex-col items-center rounded bg-gray-50 p-4 shadow";
    const highlightClass = "outline outline-blue-300";
    if (curTab()?.type == typ) {
      return className(defaultClass, highlightClass);
    } else {
      return className(defaultClass, "");
    }
  };

  return (
    <div className="ml-2 mt-6 flex flex-wrap gap-x-2 gap-y-2">
      <div
        className={toolClass("Database")}
        onClick={() => dispatch({ type: "OpenDatabase" })}
      >
        <CircleStackIcon className="h-6 w-6" />
        <div>Database</div>
      </div>
      <div className={toolClass("Reports")}>
        <ArrowTrendingUpIcon className="h-6 w-6" />
        <div>Reports</div>
      </div>
      <div className={toolClass("Logs")}>
        <Bars3Icon className="h-6 w-6" />
        <div>Logs</div>
      </div>
      <div className={toolClass("Secrets")}>
        <LockClosedIcon className="h-6 w-6" />
        <div className="text-sm">Secrets</div>
      </div>
    </div>
  );
}
