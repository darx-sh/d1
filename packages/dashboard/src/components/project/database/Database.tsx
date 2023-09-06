import { useState } from "react";
import { useEffectOnce } from "usehooks-ts";
import TableList from "~/components/project/database/TableList";
import TableDetails from "~/components/project/database/TableDetails";
import Spinner from "~/components/project/Spinner";
import { useProjectState } from "~/components/project/ProjectContext";
import {
  DatabaseProvider,
  Row,
  SchemaDef,
  TableDef,
  useDatabaseDispatch,
  useDatabaseState,
  TableDefError,
  ColumnError,
  ColumnMarkMap,
  tableChanged,
  DefaultTableTemplate,
} from "~/components/project/database/DatabaseContext";
import { env } from "~/env.mjs";
import axios, { AxiosResponse } from "axios";
import TableEditorModal from "~/components/project/database/TableEditorModal";
import CancelEditor from "~/components/project/database/CancelEditor";
import {
  CreateTableReq,
  invoke,
  invokeAsync,
  TableEditReq,
} from "~/utils/index";

import {
  MySQLFieldType,
  mysqlToFieldType,
  primitiveToDefaultValue,
} from "~/utils/types";

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

function Database() {
  const [isLoading, setIsLoading] = useState(true);
  const projectState = useProjectState();
  const dbDispatch = useDatabaseDispatch();
  const dbState = useDatabaseState();
  const envId = projectState.envInfo!.id;
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);

  useEffectOnce(() => {
    listTable();
  });

  const createTableButton = () => {
    return (
      <button
        type="button"
        className="relative mx-auto block w-1/2 rounded-lg border-2 border-dashed border-gray-300 p-12 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
        onClick={() => {
          dbDispatch({ type: "InitDraftFromTemplate" });
        }}
      >
        <svg
          className="mx-auto h-12 w-12 text-gray-400"
          stroke="currentColor"
          fill="none"
          viewBox="0 0 48 48"
          aria-hidden="true"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M8 14v20c0 4.418 7.163 8 16 8 1.381 0 2.721-.087 4-.252M8 14c0 4.418 7.163 8 16 8s16-3.582 16-8M8 14c0-4.418 7.163-8 16-8s16 3.582 16 8m0 0v14m0-4c0 4.418-7.163 8-16 8S8 28.418 8 24m32 10v6m0 0v6m0-6h6m-6 0h-6"
          />
        </svg>
        <span className="mt-2 block text-sm font-semibold text-gray-900">
          Create a new table
        </span>
      </button>
    );
  };

  const listTable = (selectTableName?: string) => {
    const req = {};
    invoke(
      envId,
      "_plugins/schema/api.listTable",
      req,
      (response) => {
        const rsp = response as ListTableRsp;
        const schema = rspToSchema(rsp);
        dbDispatch({ type: "LoadTables", schemaDef: schema });
        if (selectTableName !== undefined) {
          paginateTable(selectTableName, null, null);
        } else {
          setIsLoading(false);
        }
      },
      (err) => {
        console.error(err);
        setIsLoading(false);
      }
    );
  };

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

  const saveEdit = async () => {
    const mod = dbState.editorMod;
    if (mod === "Create") {
      createTable();
    } else if (mod === "Update") {
      await updateTable();
    }
  };

  const createTable = () => {
    const tableDef = dbState.draftTable;
    const error = validateTableDef(tableDef);
    if (error !== null) {
      dbDispatch({ type: "SetDraftError", error });
    } else {
      const req = genCreateTable(tableDef);
      console.log(req);
      setIsLoading(true);
      invoke(
        envId,
        "_plugins/schema/api.ddl",
        { req: req },
        (rsp) => {
          console.log(rsp);
          listTable();
        },
        (err) => {
          console.error(err);
          setIsLoading(false);
        }
      );
    }
  };

  const updateTable = async () => {
    const curTableName = dbState.curWorkingTable!.tableName;
    const oldTableDef = dbState.schema[curTableName]!;
    const newTableDef = dbState.draftTable;
    const marks = dbState.draftColumnMarks;
    const error = validateTableDef(newTableDef);
    if (error !== null) {
      dbDispatch({ type: "SetDraftError", error });
    } else {
      const reqs = genTableEdit(oldTableDef, newTableDef, marks);
      setIsLoading(true);
      for (const req of reqs) {
        await invokeAsync(envId, "_plugins/schema/api.ddl", {
          req: req,
        });
      }
      listTable(newTableDef.name!);
    }
  };

  const cancelEdit = () => {
    const marks = dbState.draftColumnMarks;
    let hasChanged = false;
    switch (dbState.editorMod) {
      case "Create":
        if (tableChanged(DefaultTableTemplate, dbState.draftTable, marks)) {
          hasChanged = true;
        }
        break;
      case "Update":
        const tableName = dbState.curWorkingTable!.tableName;
        if (
          tableChanged(dbState.schema[tableName]!, dbState.draftTable, marks)
        ) {
          hasChanged = true;
        }
        break;
      case "None":
        throw new Error("cancel editor in non-edit mode");
    }

    if (hasChanged) {
      setShowCancelConfirm(true);
      return;
    } else {
      dbDispatch({ type: "DeleteScratchTable" });
      return;
    }
  };

  const dropTable = (tableName: string) => {
    console.log("drop table: ", tableName);
    const req = {
      dropTable: {
        tableName,
      },
    };
    invoke(
      envId,
      "_plugins/schema/api.ddl",
      { req: req },
      (rsp) => {
        console.log(rsp);
        listTable();
      },
      (err) => {
        console.error(err);
        setIsLoading(false);
      }
    );
  };

  return (
    <>
      {isLoading ? (
        <Spinner></Spinner>
      ) : (
        <div className=" flex h-full border-2 pt-2">
          <div className="w-40 flex-none bg-white">
            <button
              type="button"
              className="ml-2 mt-2 block rounded bg-gray-400 px-2 py-2 text-left text-sm font-semibold text-white shadow-sm hover:bg-indigo-400"
              onClick={() => {
                dbDispatch({ type: "InitDraftFromTemplate" });
              }}
            >
              Create Table
            </button>
            <TableList
              onSelectTable={(tableName: string) => {
                paginateTable(tableName, null, null);
              }}
            ></TableList>
          </div>
          <div className="ml-2 min-w-0 flex-1 bg-white">
            {dbState.curWorkingTable ? (
              <TableDetails
                handleDeleteTable={(tableName: string) => {
                  setIsLoading(true);
                  dropTable(tableName);
                }}
                handleSave={saveEdit}
                handleCancel={cancelEdit}
              ></TableDetails>
            ) : (
              createTableButton()
            )}
          </div>
          <TableEditorModal
            open={dbState.editorMod === "Create"}
            handleSave={saveEdit}
            handleCancel={cancelEdit}
          ></TableEditorModal>
          <CancelEditor
            open={showCancelConfirm}
            onYes={() => {
              setShowCancelConfirm(false);
              dbDispatch({ type: "DeleteScratchTable" });
            }}
            onNo={() => {
              setShowCancelConfirm(false);
            }}
          ></CancelEditor>
        </div>
      )}
    </>
  );
}

function validateTableDef(tableDef: TableDef): TableDefError | null {
  let hasError = false;
  const tableDefErr: TableDefError = { nameError: null, columnsError: [] };

  if (tableDef.name === null || tableDef.name === "") {
    hasError = true;
    tableDefErr.nameError = "Table name cannot be empty";
  }

  tableDef.columns.forEach((col) => {
    const columnError: ColumnError = {
      nameError: null,
      fieldTypeError: null,
    };
    if (col.name === "") {
      hasError = true;
      columnError.nameError = "Column name cannot be empty";
    }
    if (col.fieldType === "NotDefined") {
      hasError = true;
      columnError.fieldTypeError = "Column type cannot be empty";
    }

    // rule 1: text field cannot have default value

    // rule 2: AUTO_INCREMENT cannot have a default value

    // rule 3: AUTO_INCREMENT cannot be nullable

    // rule 4: AUTO_INCREMENT MUST be a KEY.
    tableDefErr.columnsError.push(columnError);
  });

  if (hasError) {
    return tableDefErr;
  } else {
    return null;
  }
}

function genCreateTable(tableDef: TableDef): CreateTableReq {
  const columns = tableDef.columns.map((c) => {
    const name = c.name;
    const fieldType = c.fieldType;
    const isNullable = c.isNullable;
    const extra = c.extra;
    return {
      name,
      fieldType,
      isNullable,
      defaultValue: c.defaultValue,
      extra,
    };
  });
  const req = {
    createTable: {
      tableName: tableDef.name!,
      columns: columns,
    },
  };
  return req;
}

function genTableEdit(
  oldTable: TableDef,
  newTable: TableDef,
  marks: ColumnMarkMap
): TableEditReq[] {
  const reqs: TableEditReq[] = [];
  if (oldTable.name! !== newTable.name!) {
    reqs.push({
      renameTable: {
        oldTableName: oldTable.name!,
        newTableName: newTable.name!,
      },
    });
  }

  console.assert(
    newTable.columns.length >= oldTable.columns.length,
    "new table should have more columns than old table"
  );

  for (let i = 0; i < newTable.columns.length; i++) {
    // handle "add", "delete", "update"; ignore none.
    switch (marks[i]) {
      case undefined:
        break;
      case "None":
        break;
      case "Add":
        reqs.push({
          addColumn: {
            tableName: newTable.name!,
            column: newTable.columns[i]!,
          },
        });
        break;
      case "Update":
        // Can only rename column name now.
        // Other properties like column type, default value, nullable cannot be changed after table is created.
        if (oldTable.columns[i]!.name !== newTable.columns[i]!.name) {
          reqs.push({
            renameColumn: {
              tableName: newTable.name!,
              oldColumnName: oldTable.columns[i]!.name,
              newColumnName: newTable.columns[i]!.name,
            },
          });
        }
        break;
      case "Del":
        reqs.push({
          dropColumn: {
            tableName: newTable.name!,
            columnName: newTable.columns[i]!.name,
          },
        });
        break;
    }
  }
  return reqs;
}

export default function DatabaseWrapper() {
  return (
    <DatabaseProvider>
      <Database />
    </DatabaseProvider>
  );
}
