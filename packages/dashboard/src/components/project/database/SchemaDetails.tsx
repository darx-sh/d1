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
  console.log("schema details");
  useEffect(() => {
    console.log("schema details effect");
    const fetchData = async () => {
      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
      setIsLoading(false);
    };
    fetchData().catch(console.error);
  }, []);

  return <>{isLoading ? <Spinner /> : <div>Schema Details</div>}</>;
}
