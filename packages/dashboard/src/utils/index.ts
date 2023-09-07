import { env } from "~/env.mjs";
import axios from "axios";
import { DxColumnType } from "~/components/project/database/DatabaseContext";

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

export type DDLReq = CreateTableReq | TableEditReq;

export interface CreateTableReq {
  createTable: {
    tableName: string;
    columns: DxColumnType[];
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
