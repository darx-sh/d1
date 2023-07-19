import Directory from "~/components/project_v2/LeftDirectory";
import Tools from "~/components/project_v2/LeftTools";
export default () => {
  return (
    <div className="flex h-full flex-col">
      <div className="h-1/3 border">
        <Directory></Directory>
      </div>
      <div className="flex-1 border">
        <Tools></Tools>
      </div>
    </div>
  );
};
