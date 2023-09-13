import {
  DxColumnType,
  SchemaDef,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import {
  MySQLFieldType,
  mysqlToFieldType,
  primitiveToDefaultValue,
} from "~/utils/types";
import { Row } from "~/components/project/database/DatabaseContext";
import { env } from "~/env.mjs";
import axios from "axios";

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

export async function insertRow(envId: string, tableName: string, values: Row) {
  const res = await invokeAsync(envId, "_plugins/table/api.insertRow", {
    tableName,
    values,
  });
}

// export async function updateRow(envId: string, tableName: string, id: number, values: Row) {
//   throw new Error("not implemented");
// }

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

export type DDLReq = CreateTableReq | DropTableReq | TableEditReq;

export interface CreateTableReq {
  createTable: {
    tableName: string;
    columns: DxColumnType[];
  };
}

export interface DropTableReq {
  dropTable: {
    tableName: string;
  };
}

export type TableEditReq =
  | RenameTableReq
  | AddColumnReq
  | RenameColumnReq
  | DropColumnReq;

export interface RenameTableReq {
  renameTable: {
    oldTableName: string;
    newTableName: string;
  };
}

export interface AddColumnReq {
  addColumn: {
    tableName: string;
    column: DxColumnType;
  };
}

export interface RenameColumnReq {
  renameColumn: {
    tableName: string;
    oldColumnName: string;
    newColumnName: string;
  };
}

export interface DropColumnReq {
  dropColumn: {
    tableName: string;
    columnName: string;
  };
}

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

function invoke<T>(
  envId: string,
  path: string,
  param: T,
  success: (data: any) => void,
  error: (e: any) => void
) {
  const functionUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/${path}`;
  axios
    .post(functionUrl, param, {
      headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
    })
    .then((response) => {
      success(response.data);
    })
    .catch((e) => {
      error(e);
    });
}

async function invokeAsync<P, R>(envId: string, path: string, param: P) {
  const functionUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/${path}`;
  const response = await axios.post<R>(functionUrl, param, {
    headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
  });
  return response.data;
}
