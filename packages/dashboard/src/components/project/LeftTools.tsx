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

  return (
    <div className="ml-2 mt-6 flex flex-wrap gap-x-2 gap-y-2">
      <div
        className={className(
          curTab()?.type === "Database" ? "outline outline-blue-300" : "",
          "flex h-20 w-20 flex-col items-center rounded bg-gray-200 p-4 shadow-md"
        )}
        onClick={() => dispatch({ type: "OpenDatabase" })}
      >
        <CircleStackIcon className="h-6 w-6" />
        <div>Database</div>
      </div>
      <div className="flex h-20 w-20 flex-col items-center rounded bg-gray-200 p-4 p-4 shadow-md">
        <ArrowTrendingUpIcon className="h-6 w-6" />
        <div>Reports</div>
      </div>
      <div className="flex h-20 w-20 flex-col items-center rounded bg-gray-200 p-4 p-4 shadow-md">
        <Bars3Icon className="h-6 w-6" />
        <div>Logs</div>
      </div>
      <div className="flex h-20 w-20 flex-col items-center rounded bg-gray-200 p-4 p-4 shadow-md">
        <LockClosedIcon className="h-6 w-6" />
        <div className="text-sm">Secrets</div>
      </div>
    </div>
  );
}
