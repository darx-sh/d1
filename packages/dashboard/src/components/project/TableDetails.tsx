import { Row, useDatabaseState } from "./DatabaseContext";
import { Cog6ToothIcon } from "@heroicons/react/24/outline";
export default function TableDetails() {
  const dbState = useDatabaseState();
  const curTable = dbState.curData!.tableName;
  const columnNames = dbState.schema[curTable]!;

  const renderColumnNames = (columnNames: string[]) => {
    return (
      <tr>
        {columnNames.map((col, idx) => {
          return (
            <th
              key={col}
              scope="col"
              className="border bg-gray-300 py-3.5 pl-4 pr-8 text-left text-sm font-semibold text-gray-900"
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

  const renderOneRow = (row: Row, columnNames: string[], ridx: number) => {
    return (
      <tr key={ridx}>
        {columnNames.map((name, idx) => {
          return (
            <td
              key={idx}
              className="whitespace-nowrap border py-4 pl-4 pr-8 text-sm text-gray-500"
            >
              {row[name]}
            </td>
          );
        })}
      </tr>
    );
  };

  const renderRows = (columnNames: string[]) => {
    return dbState.curData?.rows.map((row, idx) => {
      return renderOneRow(row, columnNames, idx);
    });
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <div className="flex items-center">
            <h1 className="p-2 text-base font-semibold leading-6 text-gray-900">
              {curTable}
            </h1>
            <Cog6ToothIcon className="h-6 w-6 hover:bg-gray-600"></Cog6ToothIcon>
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
          <tbody>{renderRows(columnNames)}</tbody>
        </table>
      </div>
    </div>
  );
}
