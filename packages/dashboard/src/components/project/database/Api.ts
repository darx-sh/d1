import {
  DxColumnType,
  SchemaDef,
  TableDef,
  Row,
} from "~/components/project/database/DatabaseContext";
import {
  Datum,
  datumToPrimitive,
  MySQLFieldType,
  mysqlToFieldType,
  primitiveToDatum,
  primitiveToDefaultValue,
  PrimitiveType,
} from "~/utils/types";
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
  const rows: ApiRow[] = await invokeAsync(
    envId,
    "_plugins/table/api.paginateTable",
    {
      tableName,
      prevCreatedAt,
      prevIds,
      limit: 100,
    }
  );
  return rows;
}

export interface ApiRow {
  [key: string]: PrimitiveType;
}

export function ApiRowToRowDatum(apiRow: ApiRow, tableDef: TableDef): Row {
  const entries = Object.entries(apiRow).map(([k, v]) => {
    const columnDef = tableDef.columns.find((c) => c.name === k);
    if (!columnDef) {
      throw new Error(`column ${k} not found`);
    }
    return [k, primitiveToDatum(v, columnDef.fieldType)];
  });
  return Object.fromEntries(entries) as Row;
}

export function RowDatumToApiRow(row: Row): ApiRow {
  const entries = Object.entries(row).map(([k, v]) => {
    return [k, datumToPrimitive(v)];
  });
  return Object.fromEntries(entries) as ApiRow;
}

export async function insertRow(
  envId: string,
  tableName: string,
  values: ApiRow
) {
  const res = await invokeAsync(envId, "_plugins/table/api.insertRow", {
    tableName,
    values,
  });
}

export async function updateRow(
  envId: string,
  tableName: string,
  id: number,
  values: ApiRow
) {
  const res = await invokeAsync(envId, "_plugins/table/api.updateRow", {
    tableName,
    id,
    values,
  });
}

export async function deleteRows(
  envId: string,
  tableName: string,
  ids: number[]
) {
  const res = await invokeAsync(envId, "_plugins/table/api.deleteRows", {
    tableName,
    ids,
  });
}

export async function ddl(envId: string, req: DDLReq) {
  const rsp: object = await invokeAsync(envId, "_plugins/schema/api.ddl", {
    req: req,
  });
  return rsp;
}

type ListTableRsp = {
  tableName: string;
  columns: ApiRspColumnType[];
  primaryKey: string[];
}[];

interface ApiRspColumnType {
  columnName: string;
  fieldType: string;
  nullable: string;
  defaultValue: null | string | number | boolean;
  comment: string;
  extra: string;
}

export type DDLReq = CreateTableReq | DropTableReq | TableEditReq;

export interface CreateTableReq {
  createTable: {
    tableName: string;
    columns: ApiReqColumnType[];
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
    column: ApiReqColumnType;
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

interface ApiReqColumnType {
  name: string;
  fieldType: string;
  isNullable: boolean;
  defaultValue: ApiDefaultValue;
}

interface ApiDefaultValue {
  typ:
    | "int64"
    | "float64"
    | "bool"
    | "varchar(255)"
    | "text"
    | "datetime"
    | "expr"
    | "NULL"
    | "NotDefined";
  value: string;
}

export function columnTypeToApiReqColumnType(
  columnType: DxColumnType
): ApiReqColumnType {
  let fieldType: string;
  if (columnType.fieldType === "int64 Auto Increment") {
    fieldType = "int64Identity";
  } else if (columnType.fieldType === "Not Defined") {
    throw new Error("internal error: field type is not defined");
  } else {
    fieldType = columnType.fieldType;
  }

  let defaultValueType: ApiDefaultValue["typ"];
  if (columnType.defaultValue.typ === "Not Defined") {
    defaultValueType = "NotDefined";
  } else {
    defaultValueType = columnType.defaultValue.typ;
  }

  let defaultValueData: string;
  switch (columnType.defaultValue.typ) {
    case "NULL":
      defaultValueData = "";
      break;
    case "Not Defined":
      defaultValueData = "";
      break;
    case "int64":
      const v = columnType.defaultValue as { typ: "int64"; value: number };
      defaultValueData = v.value.toString();
      break;
    case "float64":
      const v2 = columnType.defaultValue as { typ: "float64"; value: number };
      defaultValueData = v2.value.toString();
      break;
    case "bool":
      const v3 = columnType.defaultValue as { typ: "bool"; value: boolean };
      defaultValueData = v3.value.toString();
      break;
    case "datetime":
      const v4 = columnType.defaultValue as { typ: "datetime"; value: string };
      defaultValueData = v4.value;
      break;
    case "expr":
      const v5 = columnType.defaultValue as { typ: "expr"; value: string };
      defaultValueData = v5.value;
      break;
    case "varchar(255)":
      const v6 = columnType.defaultValue as {
        typ: "varchar(255)";
        value: string;
      };
      defaultValueData = v6.value;
      break;
    case "text":
      const v7 = columnType.defaultValue as { typ: "text"; value: string };
      defaultValueData = v7.value;
      break;
  }

  return {
    name: columnType.name,
    fieldType: fieldType,
    isNullable: columnType.isNullable,
    defaultValue: {
      typ: defaultValueType,
      value: defaultValueData,
    },
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
            defaultValue: primitiveToDefaultValue(defaultValue, dxFieldType),
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
