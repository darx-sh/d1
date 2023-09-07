import { DDLReq, invoke, invokeAsync } from "~/utils";
import {
  SchemaDef,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import {
  MySQLFieldType,
  mysqlToFieldType,
  primitiveToDefaultValue,
} from "~/utils/types";
import { Row } from "~/components/project/database/DatabaseContext";

export async function loadSchema(envId: string) {
  const req = {};
  const response: ListTableRsp = await invokeAsync(
    envId,
    "_plugins/schema/api.listTable",
    { req: req }
  );
  const schema = rspToSchema(response);
  return schema;
}

export async function paginateTable(
  envId: string,
  tableName: string,
  prevCreatedAt: string | null,
  prevIds: string[] | null
) {
  const rows: Row[] = await invokeAsync(
    envId,
    "_plugins/table/api.paginateTable",
    {
      tableName,
      prevCreatedAt,
      prevIds,
      limit: 10,
    }
  );
  return rows;
}

export async function ddl(envId: string, req: DDLReq) {
  const rsp: object = await invokeAsync(envId, "_plugins/schema/api.ddl", {
    req: req,
  });
  return rsp;
}

type ListTableRsp = {
  tableName: string;
  columns: {
    columnName: string;
    fieldType: string;
    nullable: string;
    defaultValue: null | string | number | boolean;
    comment: string;
    extra: string;
  }[];
  primaryKey: string[];
}[];

function rspToSchema(rsp: ListTableRsp): SchemaDef {
  const schema = {} as SchemaDef;
  for (const { tableName, columns, primaryKey } of rsp) {
    const tableDef: TableDef = {
      name: tableName,
      columns: columns.map(
        ({ columnName, fieldType, nullable, defaultValue, extra }) => {
          const dxFieldType = mysqlToFieldType(
            fieldType as MySQLFieldType,
            extra
          );
          const isNullable = nullable === "YES";
          return {
            name: columnName,
            fieldType: dxFieldType,
            isNullable,
            defaultValue: primitiveToDefaultValue(
              defaultValue,
              dxFieldType,
              isNullable
            ),
            extra: null,
          };
        }
      ),
    };
    schema[tableName] = tableDef;
  }
  return schema;
}
