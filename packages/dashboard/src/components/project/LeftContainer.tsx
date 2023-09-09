import Directory from "~/components/project/LeftDirectory";
import LeftTools from "~/components/project/LeftTools";
export default function LeftContainer() {
  return (
    <div className="flex flex-col">
      <div className="h-80 rounded border-b">
        <Directory></Directory>
      </div>
      <div className="flex-1">
        <LeftTools></LeftTools>
      </div>
    </div>
  );
}
