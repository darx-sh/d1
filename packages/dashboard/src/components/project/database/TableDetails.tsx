import { useState, useEffect } from "react";
import { Row, useDatabaseState, useDatabaseDispatch } from "./DatabaseContext";
import SchemaEditorModal from "~/components/project/database/SchemaEditorModal";
import {
  paginateTable,
  loadSchema,
  ApiRowToRowDatum,
  deleteRows,
} from "~/components/project/database/Api";
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
  const [deletingRowIds, setDeletingRowIds] = useState<(string | number)[]>([]);

  useEffect(() => {
    const fetchData = async () => {
      setIsLoading(true);
      const apiRows = await paginateTable(
        props.envId,
        props.tableName,
        null,
        null
      );
      const rows = apiRows.map((r) => ApiRowToRowDatum(r, tableDef));
      setRows(rows);
      setIsLoading(false);
    };
    fetchData().catch(console.error);
  }, []);

  const handleDelete = (ids: (string | number)[]) => {
    setDeletingRowIds(ids);
  };

  const renderContent = () => {
    return (
      <>
        <SchemaEditorModal
          open={state.editorMode === "Update"}
          envId={props.envId}
          beforeSave={() => {
            setIsLoading(true);
          }}
          afterSave={() => {
            (async () => {
              const schema = await loadSchema(props.envId);
              dispatch({ type: "LoadSchema", schemaDef: schema });
              const apiRows = await paginateTable(
                props.envId,
                props.tableName,
                null,
                null
              );
              const tableDef = schema[props.tableName]!;
              const rows = apiRows.map((r) => ApiRowToRowDatum(r, tableDef));
              setRows(rows);
              setIsLoading(false);
            })().catch(console.error);
          }}
        ></SchemaEditorModal>
        <RowEditor
          open={state.rowEditorMode !== "None"}
          envId={props.envId}
          tableName={props.tableName}
          beforeSave={() => {
            setIsLoading(true);
          }}
          afterSave={async () => {
            const apiRows = await paginateTable(
              props.envId,
              props.tableName,
              null,
              null
            );
            const rows = apiRows.map((r) => ApiRowToRowDatum(r, tableDef));
            setRows(rows);
            dispatch({ type: "DeleteRowEditor" });
            setIsLoading(false);
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

        <DangerActionConfirm
          message={"Do you want do delete selected rows"}
          open={deletingRowIds.length > 0}
          onYes={() => {
            setIsLoading(true);
            (async () => {
              const rowIds = deletingRowIds as number[];
              await deleteRows(props.envId, props.tableName, rowIds);
              const apiRows = await paginateTable(
                props.envId,
                props.tableName,
                null,
                null
              );
              const rows = apiRows.map((r) => ApiRowToRowDatum(r, tableDef));
              setRows(rows);
              setIsLoading(false);
            })().catch(console.error);
            setDeletingRowIds([]);
          }}
          onNo={() => {
            setDeletingRowIds([]);
          }}
        ></DangerActionConfirm>

        <div className="px-8">
          <div className="mt-2 flex items-center">
            <button
              type="button"
              className="mr-2 rounded-md border bg-gray-100 px-10 py-2 text-sm font-normal text-gray-900 shadow-sm hover:bg-gray-300"
              onClick={() => {
                dispatch({ type: "InitRowEditorFromEmpty" });
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
          <TableGrid
            tableDef={tableDef}
            rows={rows}
            onDelete={handleDelete}
          ></TableGrid>
        </div>
      </>
    );
  };

  return <>{isLoading ? <Spinner /> : renderContent()}</>;
}
