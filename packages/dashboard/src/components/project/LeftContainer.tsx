import Directory from "~/components/project/LeftDirectory";
import Tools from "~/components/project/LeftTools";
export default function LeftContainer() {
  return (
    <div className="flex h-full flex-col">
      <div className="h-80 rounded border-b">
        <Directory></Directory>
      </div>
      <div className="flex-1">
        <Tools></Tools>
      </div>
    </div>
  );
}
