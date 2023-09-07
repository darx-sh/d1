import { useEffect, useState } from "react";
import { loadSchema } from "~/components/project/database/Api";
import {
  useDatabaseState,
  useDatabaseDispatch,
} from "~/components/project/database/DatabaseContext";
import Spinner from "~/components/project/Spinner";

interface SchemaDetailsProps {
  envId: string;
}

export default function SchemaDetails(props: SchemaDetailsProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    console.log("schema details effect");
    const fetchData = async () => {
      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
      setIsLoading(false);
    };
    fetchData().catch(console.error);
  }, []);

  const renderContent = () => {
    return (
      <div className="divide-y divide-gray-200 overflow-hidden rounded-lg bg-white shadow">
        <div className="px-4 py-5 sm:p-6">customer</div>
        <div className="px-4 py-4 sm:px-6">
          {/* Content goes here */}
          {/* We use less vertical padding on card footers at all sizes than on headers or body sections */}
          jjj
        </div>
      </div>
    );
  };

  return <>{isLoading ? <Spinner /> : renderContent()}</>;
}
