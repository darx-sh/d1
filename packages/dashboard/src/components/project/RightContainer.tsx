import ToolTabs from "~/components/project/ToolTabs";
import ToolContent from "~/components/project/ToolContent";

export default function RightContainer() {
  return (
    <div className="flex h-full flex-col">
      <div className="bg-gray-100">
        <ToolTabs></ToolTabs>
      </div>
      <div className="flex-1 bg-white">
        <ToolContent></ToolContent>
      </div>
    </div>
  );
}
