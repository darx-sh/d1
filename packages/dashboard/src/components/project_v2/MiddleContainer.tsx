import MiddleToolTabs from "~/components/project_v2/MiddleToolTabs";
import MiddleToolContent from "~/components/project_v2/MiddleToolContent";

export default function MiddleContainer() {
  return (
    <div className="flex h-full flex-col">
      <div className="bg-gray-100">
        <MiddleToolTabs></MiddleToolTabs>
      </div>
      <div className="flex-1 bg-white">
        <MiddleToolContent></MiddleToolContent>
      </div>
    </div>
  );
}
