import ToolTabs from "~/components/project/ToolTabs";
import ToolContent from "~/components/project/ToolContent";

export default function RightContainer() {
  return (
    <div className="relative flex h-full flex-col">
      <ToolTabs></ToolTabs>
      <ToolContent></ToolContent>
    </div>
  );
}
