import ToolTabs from "~/components/project/ToolTabs";
import ToolContent from "~/components/project/ToolContent";

export default function RightContainer() {
  return (
    <div className="flex h-full flex-col">
      <div className="bg-gray-100">
        <ToolTabs></ToolTabs>
      </div>
      <div className="min-w-0 flex-1 bg-gray-100">
        <ToolContent></ToolContent>
      </div>
    </div>
  );
}
