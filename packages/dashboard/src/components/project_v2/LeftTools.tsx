import {
  CircleStackIcon,
  ArrowTrendingUpIcon,
  WrenchIcon,
  Bars3Icon,
  LockClosedIcon,
  CloudIcon,
} from "@heroicons/react/24/outline";

export default () => {
  return (
    <div className="ml-2 flex flex-wrap">
      <div className="flex h-20 w-20 flex-col items-center p-4">
        <CircleStackIcon className="h-6 w-6" />
        <div>Database</div>
      </div>
      <div className="flex h-20 w-20 flex-col items-center p-4">
        <CloudIcon className="h-6 w-6" />
        <div>API</div>
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
};
