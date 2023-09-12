import { useState, useEffect } from "react";
import {
  Row,
  useDatabaseState,
  useDatabaseDispatch,
  TableDef,
} from "./DatabaseContext";
import SchemaEditorModal from "~/components/project/database/SchemaEditorModal";
import { displayColumnValue } from "~/utils/types";
import { paginateTable, loadSchema } from "~/components/project/database/Api";
import Spinner from "~/components/project/Spinner";
import RowEditor from "~/components/project/database/RowEditor";
import DangerActionConfirm from "~/components/project/database/DangerActionConfirm";
import TableActions from "~/components/project/database/TableActions";
import TableGrid from "~/components/project/database/TableGrid";

export interface TableDetailsProps {
  envId: string;
  tableName: string;
}

export default function TableDetails(props: TableDetailsProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const tableDef = state.schema[props.tableName]!;
  const [isLoading, setIsLoading] = useState(true);
  const [rows, setRows] = useState<Row[]>([]);
  const [showCancelInsertConfirm, setShowCancelInsertConfirm] = useState(false);

  const columnNames = tableDef.columns.map((c) => {
    if (c.name === null) {
      throw new Error("Column name cannot be null");
    }
    return c.name;
  });

  useEffect(() => {
    const fetchData = async () => {
      setIsLoading(true);
      const rows = await paginateTable(
        props.envId,
        props.tableName,
        null,
        null
      );
      setIsLoading(false);
      setRows(rows);
    };
    fetchData().catch(console.error);
  }, []);

  const renderColumnNames = (columnNames: string[]) => {
    return (
      <tr>
        {columnNames.map((col, idx) => {
          return (
            <th
              key={col}
              scope="col"
              className="border bg-gray-50 px-4 py-3.5 text-left text-sm font-normal text-gray-900"
            >
              {col}
            </th>
          );
        })}
        <th scope="col" className="relative py-3.5 pl-3 pr-4 sm:pr-0">
          <span className="sr-only">Edit</span>
        </th>
      </tr>
    );
  };

  const renderOneRow = (
    row: Row,
    columnNames: string[],
    ridx: number,
    tableDef: TableDef
  ) => {
    return (
      <tr key={ridx}>
        {columnNames.map((name, idx) => {
          return (
            <td
              key={idx}
              className="whitespace-nowrap border px-4 py-4 text-sm text-gray-500 hover:cursor-pointer hover:bg-gray-200"
            >
              {displayColumnValue(row[name]!, tableDef.columns[idx]!.fieldType)}
            </td>
          );
        })}
      </tr>
    );
  };

  const renderRows = (columnNames: string[], tableDef: TableDef) => {
    return rows.map((row, idx) => {
      return renderOneRow(row, columnNames, idx, tableDef);
    });
  };

  const renderContent = () => {
    return (
      <>
        <SchemaEditorModal
          open={state.editorMod === "Update"}
          envId={props.envId}
          beforeSave={() => {
            setIsLoading(true);
          }}
          afterSave={() => {
            (async () => {
              const schema = await loadSchema(props.envId);
              dispatch({ type: "LoadSchema", schemaDef: schema });
              const rows = await paginateTable(
                props.envId,
                props.tableName,
                null,
                null
              );
              setRows(rows);
              setIsLoading(false);
            })().catch(console.error);
          }}
        ></SchemaEditorModal>
        <RowEditor
          open={state.draftRowMod !== "None"}
          envId={props.envId}
          tableName={props.tableName}
          beforeSave={() => {
            console.log("beforeSave");
          }}
          afterSave={() => {
            console.log("afterSave");
          }}
        ></RowEditor>
        <DangerActionConfirm
          message={"Do you want to discard the row"}
          open={showCancelInsertConfirm}
          onYes={() => {
            dispatch({ type: "DeleteRowEditor" });
            setShowCancelInsertConfirm(false);
          }}
          onNo={() => setShowCancelInsertConfirm(false)}
        ></DangerActionConfirm>

        <div className="px-8">
          <div className="mt-2 flex items-center">
            <button
              type="button"
              className="mr-2 rounded-md border bg-gray-100 px-10 py-2 text-sm font-normal text-gray-900 shadow-sm hover:bg-gray-300"
              onClick={() => {
                dispatch({ type: "InitRowEditorFromTemplate" });
              }}
            >
              New Record
            </button>
            <TableActions
              onEdit={() => {
                dispatch({
                  type: "InitDraftFromTable",
                  tableName: tableDef.name!,
                });
              }}
              onDelete={null}
            ></TableActions>
          </div>
          <TableGrid tableDef={tableDef} rows={rows}></TableGrid>
          {/*<div className="overflow-auto py-2 align-middle">*/}
          {/*  <table>*/}
          {/*    <thead>{renderColumnNames(columnNames)}</thead>*/}
          {/*    <tbody>{renderRows(columnNames, tableDef)}</tbody>*/}
          {/*  </table>*/}
          {/*</div>*/}
        </div>
      </>
    );
  };

  return <>{isLoading ? <Spinner /> : renderContent()}</>;
}
