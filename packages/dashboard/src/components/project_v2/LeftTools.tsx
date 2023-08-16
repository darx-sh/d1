import {
  CircleStackIcon,
  ArrowTrendingUpIcon,
  Bars3Icon,
  LockClosedIcon,
} from "@heroicons/react/24/outline";
import { useProjectDispatch } from "./ProjectContext";

export default function LeftTools() {
  const projectDispatch = useProjectDispatch()!;
  return (
    <div className="ml-2 flex flex-wrap">
      <div
        className="flex h-20 w-20 flex-col items-center p-4"
        onClick={() => projectDispatch({ type: "OpenDatabase" })}
      >
        <CircleStackIcon className="h-6 w-6" />
        <div>Database</div>
      </div>
      <div className="flex h-20 w-20 flex-col items-center p-4">
        <ArrowTrendingUpIcon className="h-6 w-6" />
        <div>Reports</div>
      </div>
      <div className="flex h-20 w-20 flex-col items-center p-4">
        <Bars3Icon className="h-6 w-6" />
        <div>Logs</div>
      </div>
      <div className="flex h-20 w-20 flex-col items-center p-4">
        <LockClosedIcon className="h-6 w-6" />
        <div className="text-sm">Secrets</div>
      </div>
    </div>
  );
}
