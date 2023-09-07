import { useEffect, useState } from "react";
import { loadSchema } from "~/components/project/database/Api";
import {
  useDatabaseState,
  useDatabaseDispatch,
  DxColumnType,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import Spinner from "~/components/project/Spinner";
import { displayDefaultValue } from "~/utils/types";

interface SchemaDetailsProps {
  envId: string;
}

export default function SchemaDetails(props: SchemaDetailsProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
      setIsLoading(false);
    };
    fetchData().catch(console.error);
  }, []);

  const renderColumn = (column: DxColumnType) => {
    return <div key={column.name}>{column.name}</div>;
  };
  const renderTable = (tableDef: TableDef) => {
    return (
      <div
        className="w-56 divide-y divide-gray-200 overflow-hidden rounded-lg bg-gray-50 shadow"
        key={tableDef.name}
      >
        <div className="px-4 py-5 sm:p-6">{tableDef.name}</div>
        <div className="px-4 py-4 sm:px-6">
          {tableDef.columns.map((column) => {
            return renderColumn(column);
          })}
        </div>
      </div>
    );
  };

  const renderContent = () => {
    const a = Object.entries(state.schema);
    return (
      <>
        <div className="flex flex-wrap gap-x-8 gap-y-8 px-40 py-10">
          {Object.entries(state.schema).map(([_, tableDef]) => {
            return renderTable(tableDef);
          })}
        </div>
      </>
    );
  };

  return <>{isLoading ? <Spinner /> : renderContent()}</>;
}
