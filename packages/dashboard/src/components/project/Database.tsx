import { useState } from "react";
import { useEffectOnce } from "usehooks-ts";
import TableList from "~/components/project/TableList";
import TableDetails from "~/components/project/TableDetails";
import Spinner from "~/components/project/Spinner";
import { useProjectState } from "~/components/project/ProjectContext";
import {
  DatabaseProvider,
  Row,
  SchemaDef,
  TableDef,
  FieldType,
  useDatabaseDispatch,
  DefaultValueType,
} from "~/components/project/DatabaseContext";
import { env } from "~/env.mjs";
import axios from "axios";

type ListTableRsp = {
  tableName: string;
  columns: {
    columnName: string;
    fieldType: string;
    nullable: string;
    defaultValue: DefaultValueType;
    comment: string;
  }[];
  primaryKey: string[];
}[];

function rspToSchema(rsp: ListTableRsp): SchemaDef {
  const schema = {} as SchemaDef;
  for (const { tableName, columns, primaryKey } of rsp) {
    const tableDef: TableDef = {
      name: tableName,
      columns: columns.map(
        ({ columnName, fieldType, nullable, defaultValue }) => {
          return {
            name: columnName,
            fieldType: fieldType.toLowerCase() as FieldType,
            isNullable: nullable === "YES",
            defaultValue,
            isPrimary: primaryKey.includes(columnName),
          };
        }
      ),
    };
    schema[tableName] = tableDef;
  }
  return schema;
}

function Database() {
  const [isLoading, setIsLoading] = useState(true);
  const projectState = useProjectState();
  const dbDispatch = useDatabaseDispatch();
  const envId = projectState.envInfo!.id;

  const paginateTable = (
    tableName: string,
    prevCreatedAt: string | null,
    prevIds: string[] | null
  ) => {
    const paginateTableUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/_plugins/table/api.paginateTable`;
    axios
      .post(
        paginateTableUrl,
        { tableName, prevCreatedAt, prevIds, limit: 10 },
        {
          headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
        }
      )
      .then((response) => {
        const rows = response.data as Row[];
        dbDispatch({ type: "LoadData", tableName, rows });
        setIsLoading(false);
      })
      .catch((error) => console.log("paginateTable error: ", error));
  };

  useEffectOnce(() => {
    const listTableUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/_plugins/schema/api.listTable`;
    axios
      .post(
        listTableUrl,
        {},
        {
          headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
        }
      )
      .then((response) => {
        const rsp = response.data as ListTableRsp;
        const schema = rspToSchema(rsp);
        dbDispatch({ type: "LoadTables", schemaDef: schema });
        if (rsp.length > 0) {
          paginateTable(rsp[0]!.tableName, null, null);
        } else {
          setIsLoading(false);
        }
      })
      .catch((error) => console.log("invoke function error: ", error));
  });

  return (
    <>
      {isLoading ? (
        <Spinner></Spinner>
      ) : (
        <div className=" flex h-full border-2 pt-2">
          <div className="w-40 flex-none bg-white">
            <TableList></TableList>
          </div>
          <div className="ml-2 min-w-0 flex-1 bg-white">
            <TableDetails></TableDetails>
          </div>
        </div>
      )}
    </>
  );
}

export default function DatabaseWrapper() {
  return (
    <DatabaseProvider>
      <Database />
    </DatabaseProvider>
  );
}
