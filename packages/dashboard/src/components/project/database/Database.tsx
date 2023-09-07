import { DatabaseProvider } from "~/components/project/database/DatabaseContext";
import { useProjectState } from "~/components/project/ProjectContext";
import DatabaseNav from "~/components/project/database/DatabaseNav";
import DatabaseDetails from "~/components/project/database/DatabaseDetails";

function Database() {
  const projectState = useProjectState();
  const envId = projectState.envInfo!.id;

  return (
    <div className=" flex h-full border-2 pt-2">
      <div className="w-40 flex-none bg-white">
        <DatabaseNav></DatabaseNav>
      </div>
      <div className="ml-2 min-w-0 flex-1 bg-white">
        <DatabaseDetails envId={envId} />
      </div>
    </div>
  );
}

export default function DatabaseWrapper() {
  return (
    <DatabaseProvider>
      <Database />
    </DatabaseProvider>
  );
}
