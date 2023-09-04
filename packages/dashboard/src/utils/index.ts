import { env } from "~/env.mjs";
import axios from "axios";
import {
  defaultValueToJSON,
  DxColumnType,
  DxFieldType,
} from "~/components/project/database/DatabaseContext";

export function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

export function invoke<T>(
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

export async function invokeAsync<P, R>(envId: string, path: string, param: P) {
  const functionUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/${path}`;
  const response = await axios.post<R>(functionUrl, param, {
    headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
  });
  return response.data;
}

export interface CreateTableReq {
  createTable: {
    tableName: string;
    columns: DxColumnJsonType[];
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
    column: DxColumnJsonType;
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

// DxColumnJsonType corresponds to the type of the backend defined DxColumnType.
interface DxColumnJsonType {
  name: string;
  fieldType: DxFieldType;
  defaultValue: DxDefaultJsonType | null;
  isNullable: boolean;
  extra: string | null;
}

export type DxDefaultJsonType =
  | { int64: number }
  | { float64: number }
  | { bool: boolean }
  | { text: string }
  | { datetime: string }
  | { expr: string }
  | "NULL";

export function columnTypeToJson(c: DxColumnType): DxColumnJsonType {
  console.assert(c.name !== null, "column name is null");
  console.assert(c.fieldType !== null, "column fieldType is null");

  return {
    name: c.name!,
    fieldType: c.fieldType!,
    defaultValue:
      c.defaultValue === null ? null : defaultValueToJSON(c.defaultValue),
    isNullable: c.isNullable,
    extra: c.extra,
  };
}

export type PrimitiveTypes = number | string | boolean | null;
