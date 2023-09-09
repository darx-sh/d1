import { useEffect, useState } from "react";
import { loadSchema } from "~/components/project/database/Api";
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
  TableCellsIcon,
  DocumentTextIcon,
  FingerPrintIcon,
  MinusIcon,
  PencilIcon,
  PencilSquareIcon,
} from "@heroicons/react/20/solid";
import className from "classnames";

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
    const renderIcon = () => {
      return null;
      // switch (column.fieldType) {
      //   case "datetime":
      //     return <CalendarDaysIcon className={iconClass}></CalendarDaysIcon>;
      //   case "text":
      //     return <DocumentTextIcon className={iconClass}></DocumentTextIcon>;
      //   case "int64Identity":
      //     return <FingerPrintIcon className={iconClass}></FingerPrintIcon>;
      //   default:
      //     return <MinusIcon className={iconClass}></MinusIcon>;
      // }
    };

    return (
      <div key={column.name} className="flex items-center p-2.5">
        {renderIcon()}
        <div className="px-2 text-sm"> {column.name}</div>
        <div className="ml-auto flex items-center">
          <PencilSquareIcon
            className={className(iconClass, "mr-6")}
          ></PencilSquareIcon>
          <div className="w-20 rounded-lg bg-gray-200 p-1 text-center text-xs">
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
          <TableCellsIcon className="h-6 w-6 text-indigo-300"></TableCellsIcon>
          <div className="px-2">{tableDef.name}</div>
        </div>
        {tableDef.columns.map((column) => {
          return renderColumn(column);
        })}
        <a className="flex items-center py-2 text-indigo-400">
          <PlusSmallIcon className="h-6 w-6">Add Column</PlusSmallIcon>
          <div className=""> Add Column</div>
        </a>
      </div>
    );
  };

  const renderContent = () => {
    const a = Object.entries(state.schema);
    return (
      <>
        <button
          type="button"
          className="ml-10 mt-2 rounded-md bg-indigo-500 px-10 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
        >
          New Table
        </button>
        <div className="grid grid-cols-3 gap-x-6 gap-y-8 px-10 py-5">
          {Object.entries(state.schema).map(([_, tableDef]) => {
            return renderTable(tableDef);
          })}
        </div>
      </>
    );
  };

  return <>{isLoading ? <Spinner /> : renderContent()}</>;
}
