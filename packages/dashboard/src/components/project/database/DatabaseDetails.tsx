import { useDatabaseState } from "~/components/project/database/DatabaseContext";
import SchemaDetails from "~/components/project/database/SchemaDetails";
import TableDetails from "~/components/project/database/TableDetails";

interface DatabaseContentProps {
  envId: string;
}

export default function DatabaseDetails(props: DatabaseContentProps) {
  const state = useDatabaseState();

  switch (state.curNav.typ) {
    case "Schema":
      return <SchemaDetails envId={props.envId} />;
    case "Table":
      return (
        <TableDetails
          envId={props.envId}
          tableName={state.curNav.tableName}
          key={state.curNav.tableName}
        />
      );
  }
}
