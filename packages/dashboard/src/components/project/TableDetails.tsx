import { Row, useDatabaseState } from "./DatabaseContext";

const people = [
  {
    name: "Lindsay Walton",
    title: "Front-end Developer",
    email: "lindsay.walton@example.com",
    role: "Member",
  },
  // More people...
];
export default function TableDetails() {
  const dbState = useDatabaseState();
  const curTable = dbState.curData!.tableName;
  const columnNames = dbState.schema[curTable]!.slice(0, 3);

  const renderColumnNames = (columnNames: string[]) => {
    return (
      <tr>
        {columnNames.map((col, idx) => {
          return (
            <th
              key={col}
              scope="col"
              className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
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
              className="whitespace-nowrap px-3 py-4 text-sm text-gray-500"
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
          <h1 className="text-base font-semibold leading-6 text-gray-900">
            {curTable}
          </h1>
        </div>
        <div className="mt-4 sm:ml-16 sm:mt-0 sm:flex-none">
          <button
            type="button"
            className="block rounded bg-gray-600 px-3 py-2 text-center text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
          >
            Edit Table
          </button>
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
        <table className="divide-y divide-gray-300">
          <thead>{renderColumnNames(columnNames)}</thead>
          <tbody className="divide-y divide-gray-200">
            {renderRows(columnNames)}
          </tbody>
        </table>
      </div>
    </div>
  );
}
