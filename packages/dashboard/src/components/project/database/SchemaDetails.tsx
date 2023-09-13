import { useEffect, useState } from "react";
import { loadSchema, ddl } from "~/components/project/database/Api";
import {
  useDatabaseState,
  useDatabaseDispatch,
  DxColumnType,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import Spinner from "~/components/project/Spinner";
import { displayFieldType } from "~/utils/types";
import SchemaEditorModal from "~/components/project/database/SchemaEditorModal";
import TableActions from "~/components/project/database/TableActions";
import DangerActionConfirm from "~/components/project/database/DangerActionConfirm";

interface SchemaDetailsProps {
  envId: string;
}

export default function SchemaDetails(props: SchemaDetailsProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const [isLoading, setIsLoading] = useState(true);
  const [deletingTableName, setDeletingTableName] = useState<null | string>(
    null
  );

  useEffect(() => {
    const fetchData = async () => {
      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
      setIsLoading(false);
    };
    fetchData().catch(console.error);
  }, []);

  const dropTable = (tableName: string | null) => {
    console.assert(tableName !== null);

    const doIt = async () => {
      setIsLoading(true);
      const rsp = await ddl(props.envId, {
        dropTable: { tableName: tableName! },
      });

      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
      setIsLoading(false);
    };
    doIt().catch(console.error);
  };

  const renderColumn = (tableName: string, column: DxColumnType) => {
    return (
      <div key={column.name} className="flex items-center p-2.5">
        <div className="px-2 text-sm"> {column.name}</div>
        <div className="ml-auto flex items-center">
          <div className="w-20 rounded-lg bg-blue-50 p-1 text-center text-xs">
            {displayFieldType(column.fieldType)}
          </div>
        </div>
      </div>
    );
  };
  const renderTable = (tableDef: TableDef) => {
    return (
      <div
        className="divide-y divide-gray-200 rounded border shadow"
        key={tableDef.name}
      >
        <div className="flex justify-between bg-gray-100 px-2 py-2 text-base font-medium text-gray-900">
          <div className="px-2">{tableDef.name}</div>
          <TableActions
            onEdit={() => {
              dispatch({
                type: "InitDraftFromTable",
                tableName: tableDef.name!,
              });
            }}
            onDelete={() => {
              setDeletingTableName(tableDef.name);
            }}
          ></TableActions>
        </div>
        {tableDef.columns.map((column) => {
          return renderColumn(tableDef.name!, column);
        })}
      </div>
    );
  };

  const DeleteTableConfirm = (tableName: string) => {
    return (
      <div>
        Do you really want to delete table{" "}
        <span className="text-2xl text-red-600">{tableName}</span>
        {" ?"}
      </div>
    );
  };
  const renderContent = () => {
    return (
      <>
        <div className="flex px-10">
          <button
            type="button"
            className="mt-2 rounded-md border bg-gray-100 px-10 py-2 text-sm font-normal text-gray-900 shadow-sm hover:bg-gray-300"
            onClick={() => {
              dispatch({ type: "InitDraftFromTemplate" });
            }}
          >
            New Table
          </button>
        </div>
        <div className="grid grid-cols-3 gap-x-6 gap-y-8 px-10 py-5">
          {Object.entries(state.schema).map(([_, tableDef]) => {
            return renderTable(tableDef);
          })}
        </div>

        <SchemaEditorModal
          open={state.editorMode !== "None"}
          envId={props.envId}
          beforeSave={() => {
            setIsLoading(true);
          }}
          afterSave={() => {
            (async () => {
              const schema = await loadSchema(props.envId);
              dispatch({ type: "LoadSchema", schemaDef: schema });
              setIsLoading(false);
            })().catch(console.error);
          }}
        ></SchemaEditorModal>
        <DangerActionConfirm
          message={DeleteTableConfirm(deletingTableName!)}
          open={deletingTableName !== null}
          onYes={() => {
            dropTable(deletingTableName);
            setDeletingTableName(null);
          }}
          onNo={() => {
            setDeletingTableName(null);
          }}
        ></DangerActionConfirm>
      </>
    );
  };

  return <>{isLoading ? <Spinner /> : renderContent()}</>;
}
