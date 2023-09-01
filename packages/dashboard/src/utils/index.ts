import { env } from "~/env.mjs";
import axios from "axios";
import { DxFieldType } from "~/components/project/DatabaseContext";

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
  console.log("invoke params: ", param);
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

export interface CreateTableReq {
  createTable: {
    tableName: string;
    columns: {
      name: string;
      fieldType: DxFieldType;
      defaultValue: DxDatumJsonType | null;
      isNullable: boolean;
      extra: string | null;
    }[];
  };
}

export type DxDatumJsonType =
  | { int64: number }
  | { float64: number }
  | { bool: boolean }
  | { text: string }
  | { datetime: string }
  | { expr: string }
  | "NULL";

export type PrimitiveTypes = number | string | boolean | null;
