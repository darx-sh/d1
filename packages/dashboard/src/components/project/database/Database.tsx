import { useDatabaseState } from "~/components/project/database/DatabaseContext";
import { useProjectState } from "~/components/project/ProjectContext";
import DatabaseNav from "~/components/project/database/DatabaseNav";
import DatabaseDetails from "~/components/project/database/DatabaseDetails";

export default function Database() {
  const projectState = useProjectState();
  const envId = projectState.envInfo!.id;

  return (
    <>
      <div className="absolute bottom-0 left-0 top-10 w-40 flex-none border-r">
        <DatabaseNav></DatabaseNav>
      </div>
      <div className="absolute bottom-0 left-40 right-0 top-10 min-w-0  overflow-auto">
        <DatabaseDetails envId={envId} />
      </div>
    </>
  );
}
