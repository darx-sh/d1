import {
  Row,
  useDatabaseState,
  useDatabaseDispatch,
  TableDef,
} from "./DatabaseContext";
import TableEditorModal from "~/components/project/database/TableEditorModal";
import TableActions from "~/components/project/database/TableActions";
import { FieldType } from "~/utils/types";

export interface TableDetailsProps {
  handleDeleteTable: (tableName: string) => void;
  handleSave: () => void;
  handleCancel: () => void;
}

export default function TableDetails(props: TableDetailsProps) {
  const dbState = useDatabaseState();
  const dbDispatch = useDatabaseDispatch();
  const curTable = dbState.curWorkingTable!.tableName;
  const tableDef = dbState.schema[curTable]!;
  const columnNames = dbState.schema[curTable]!.columns.map((c) => {
    if (c.name === null) {
      throw new Error("Column name cannot be null");
    }
    return c.name;
  });

  const renderColumnNames = (columnNames: string[]) => {
    return (
      <tr>
        {columnNames.map((col, idx) => {
          return (
            <th
              key={col}
              scope="col"
              className="border bg-gray-300 px-4 py-3.5 text-left text-sm font-semibold text-gray-900"
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

  const displayColumnValue = (v: any, fieldType: FieldType) => {
    if (v === null) {
      return "NULL";
    }

    switch (fieldType) {
      case "int64":
        return (v as number).toString();
      case "int64Identity":
        return (v as number).toString();
      case "float64":
        return (v as number).toString();
      case "bool":
        return (v as boolean).toString();
      case "datetime":
        return v as string;
      case "varchar(255)":
        return v as string;
      case "text":
        return v as string;
      case "NotDefined":
        throw new Error("Field type is not defined");
    }
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
              className="whitespace-nowrap border px-4 py-4 text-sm text-gray-500"
            >
              {displayColumnValue(row[name]!, tableDef.columns[idx]!.fieldType)}
            </td>
          );
        })}
      </tr>
    );
  };

  const renderRows = (columnNames: string[], tableDef: TableDef) => {
    return dbState.curWorkingTable?.rows.map((row, idx) => {
      return renderOneRow(row, columnNames, idx, tableDef);
    });
  };

  return (
    <>
      <TableEditorModal
        open={dbState.editorMod === "Update"}
        handleSave={props.handleSave}
        handleCancel={props.handleCancel}
      ></TableEditorModal>

      <div className="px-4 sm:px-6 lg:px-8">
        <div className="sm:flex sm:items-center">
          <div className="sm:flex-auto">
            <div className="flex items-center">
              <h1 className="mr-8 p-2 text-base font-semibold leading-6 text-gray-900">
                {curTable}
              </h1>
              <TableActions
                onEdit={() => {
                  dbDispatch({
                    type: "InitDraftFromTable",
                    tableName: curTable,
                  });
                }}
                onDelete={() => {
                  props.handleDeleteTable(curTable);
                }}
              ></TableActions>
            </div>
          </div>
          <div className="mt-4 sm:ml-16 sm:mt-0 sm:flex-none">
            <button
              type="button"
              className="block rounded bg-gray-600 px-3 py-2 text-center text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
            >
              New Record
            </button>
          </div>
        </div>

        <div className="overflow-auto py-2 align-middle">
          <table>
            <thead>{renderColumnNames(columnNames)}</thead>
            <tbody>{renderRows(columnNames, tableDef)}</tbody>
          </table>
        </div>
      </div>
    </>
  );
}
