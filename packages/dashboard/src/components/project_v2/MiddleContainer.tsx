import MiddleToolTabs from "~/components/project_v2/MiddleToolTabs";
import MiddleToolContent from "~/components/project_v2/MiddleToolContent";

export default () => {
  return (
    <>
      <div className="h-11 border">
        <MiddleToolTabs></MiddleToolTabs>
      </div>
      <div className="h-full border">
        <MiddleToolContent></MiddleToolContent>
      </div>
    </>
  );
};
