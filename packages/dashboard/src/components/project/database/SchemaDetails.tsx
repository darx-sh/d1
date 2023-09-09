import { useEffect, useState } from "react";
import { loadSchema, paginateTable } from "~/components/project/database/Api";
import {
  useDatabaseState,
  useDatabaseDispatch,
  DxColumnType,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import Spinner from "~/components/project/Spinner";
import { displayFieldType } from "~/utils/types";
import {
  CalendarDaysIcon,
  PlusSmallIcon,
  EllipsisHorizontalIcon,
  TableCellsIcon,
  DocumentTextIcon,
  FingerPrintIcon,
  MinusIcon,
  PencilIcon,
  PencilSquareIcon,
} from "@heroicons/react/20/solid";
import className from "classnames";
import TableEditorModal from "~/components/project/database/TableEditorModal";
import ColumnActionDropDown from "~/components/project/database/ColumnActionDropdown";

interface SchemaDetailsProps {
  envId: string;
}

export default function SchemaDetails(props: SchemaDetailsProps) {
  const state = useDatabaseState();
  const dispatch = useDatabaseDispatch();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      const schema = await loadSchema(props.envId);
      dispatch({ type: "LoadSchema", schemaDef: schema });
      setIsLoading(false);
    };
    fetchData().catch(console.error);
  }, []);

  const renderColumn = (column: DxColumnType) => {
    const iconClass = "h-6 w-6 text-gray-400";

    return (
      <div key={column.name} className="flex items-center p-2.5">
        <div className="px-2 text-sm"> {column.name}</div>
        <div className="ml-auto flex items-center">
          <ColumnActionDropDown></ColumnActionDropDown>
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
        <div className="flex items-center bg-gray-100 px-2 py-2 text-base font-medium text-gray-900">
          {/*<TableCellsIcon className="h-6 w-6 text-indigo-300"></TableCellsIcon>*/}
          <div className="px-2">{tableDef.name}</div>
        </div>
        {tableDef.columns.map((column) => {
          return renderColumn(column);
        })}
        <a className="flex items-center py-2 text-blue-500">
          <PlusSmallIcon className="h-6 w-6">Add Column</PlusSmallIcon>
          <div className=""> Add Column</div>
        </a>
      </div>
    );
  };

  const renderContent = () => {
    return (
      <>
        <div className="flex px-10">
          <button
            type="button"
            className=" mt-2 rounded-md bg-gray-300 px-10 py-2 text-sm font-normal  shadow-sm hover:bg-gray-400"
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

        <TableEditorModal
          open={state.editorMod === "Create"}
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
        ></TableEditorModal>
      </>
    );
  };

  return <>{isLoading ? <Spinner /> : renderContent()}</>;
}
