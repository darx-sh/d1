import { ArchiveBoxXMarkIcon, Bars3Icon } from "@heroicons/react/24/outline";
import ColumnTypeSelect from "~/components/project/ColumnTypeSelect";
import {
  ColumnDef,
  displayDefaultValue,
  useDatabaseState,
  useDatabaseDispatch,
} from "~/components/project/DatabaseContext";
import classNames from "classnames";

export default function ColumnsEditor() {
  const headerClass = "px-2 text-sm font-light italic text-center";
  const rowDataClass = "px-2 py-2 text-sm font-normal text-center";
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();

  const tableName = state.scratchTable.name ?? "";
  const columns = state.scratchTable.columns;
  const columnMarks = state.columnMarks;

  const renderColumn = (column: ColumnDef, columnIndex: number) => {
    const mark = columnMarks[columnIndex];
    if (mark === "Del") {
      return null;
    }

    const columnName = column.name ?? "Column Name";

    return (
      <tr className="hover:bg-gray-200" key={columnIndex}>
        <td className={rowDataClass}>
          <input
            type="text"
            className="block w-32 rounded-md border-0 py-1.5 pl-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
            placeholder={columnName}
            value={columnName}
            onChange={(event) => {
              dispatch({
                type: "UpdateColumn",
                columnIndex: columnIndex,
                column: {
                  ...column,
                  name: event.target.value,
                },
              });
            }}
          />
        </td>
        <td className={classNames(rowDataClass, "w-28")}>
          <ColumnTypeSelect columnType={column.fieldType}></ColumnTypeSelect>
        </td>
        <td className={rowDataClass}>
          <input
            type="text"
            name="defaultValue"
            id="defaultValue"
            className="block w-28 rounded-md border-0 py-1.5 pl-2 text-xs text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
            placeholder={displayDefaultValue(column.defaultValue)}
          />
        </td>
        <td className={rowDataClass}>
          <input
            id="primary"
            aria-describedby="comments-description"
            name="primary"
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
            checked={column.isPrimary}
            onChange={(event) => {
              dispatch({
                type: "UpdateColumn",
                columnIndex: columnIndex,
                column: {
                  ...column,
                  isPrimary: event.target.checked,
                },
              });
            }}
          />
        </td>
        <td className={rowDataClass}>
          <input
            id="isNullable"
            aria-describedby="comments-description"
            name="isNullable"
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
            checked={column.isNullable}
            onChange={(event) => {
              dispatch({
                type: "UpdateColumn",
                columnIndex: columnIndex,
                column: {
                  ...column,
                  isNullable: event.target.checked,
                },
              });
            }}
          />
        </td>
        <td
          className={rowDataClass}
          onClick={() => {
            dispatch({
              type: "DelColumn",
              columnIndex: columnIndex,
            });
          }}
        >
          <ArchiveBoxXMarkIcon
            className="-mt-2 h-6 w-6 hover:bg-gray-500"
            aria-hidden="true"
          />
        </td>
      </tr>
    );
  };

  return (
    <div>
      <table>
        <thead>
          <tr>
            <th scope="col" className={headerClass}>
              Name
            </th>
            <th scope="col" className={headerClass}>
              Type
            </th>
            <th scope="col" className={headerClass}>
              Default Value
            </th>
            <th scope="col" className={headerClass}>
              Primary
            </th>
            <th scope="col" className={headerClass}>
              Is Nullable
            </th>
            <th scope="col" className={headerClass}></th>
          </tr>
        </thead>
        <tbody>
          {columns.map((c, index) => {
            return renderColumn(c, index);
          })}
        </tbody>
      </table>
      <button
        type="button"
        className="mx-auto mt-2 block w-80 rounded-md bg-gray-600 px-16 py-1.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
        onClick={() => {
          dispatch({
            type: "AddColumn",
            column: {
              name: null,
              fieldType: null,
              defaultValue: null,
              isNullable: true,
              isPrimary: false,
            },
          });
        }}
      >
        Add Column
      </button>
    </div>
  );
}
