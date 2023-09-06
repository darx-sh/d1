import { ArchiveBoxXMarkIcon, Bars3Icon } from "@heroicons/react/24/outline";
import ColumnTypeSelect from "~/components/project/database/ColumnTypeSelect";
import {
  DxColumnType,
  displayDxDefaultValue,
  useDatabaseState,
  useDatabaseDispatch,
  DxFieldType,
} from "~/components/project/database/DatabaseContext";
import classNames from "classnames";

export default function ColumnsEditor() {
  const headerClass = "px-2 text-sm font-light italic text-center";
  const rowDataClass = "px-2 py-2 text-sm font-normal text-center";
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();

  const columns = state.draftTable.columns;
  const columnMarks = state.draftColumnMarks;

  const disableColumnProperties = (columnIndex: number) => {
    const mark = columnMarks[columnIndex];
    if (mark === "Add") {
      return false;
    } else {
      return true;
    }
  };

  const renderColumn = (column: DxColumnType, columnIndex: number) => {
    const mark = columnMarks[columnIndex];
    if (mark === "Del" || mark === "None") {
      return null;
    }

    const columnName = column.name ?? "";

    const columnReadOnly = disableColumnProperties(columnIndex);

    return (
      <tr
        className={classNames(
          columnReadOnly ? "bg-gray-100" : "hover:bg-gray-200",
          ""
        )}
        key={columnIndex}
      >
        <td className={rowDataClass}>
          <input
            type="text"
            className={classNames(
              columnReadOnly ? "" : "",
              "block w-32 rounded-md border-0 py-1.5 pl-2  text-sm text-gray-400 shadow-sm ring-1 ring-inset ring-gray-300 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
            )}
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
            disabled={columnReadOnly}
          />
        </td>
        <td className={classNames(rowDataClass, "w-44")}>
          <ColumnTypeSelect
            fieldType={column.fieldType}
            onSelect={(t: DxFieldType) => {
              dispatch({
                type: "UpdateColumn",
                columnIndex,
                column: {
                  ...column,
                  fieldType: t,
                },
              });
            }}
            disabled={columnReadOnly}
          ></ColumnTypeSelect>
        </td>
        <td className={rowDataClass}>
          <input
            type="text"
            name="defaultValue"
            id="defaultValue"
            className="block w-44 rounded-md border-0 py-1.5 pl-2 text-xs text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
            placeholder={displayDxDefaultValue(
              column.fieldType,
              column.defaultValue
            )}
            disabled={columnReadOnly ? true : false}
          />
        </td>
        <td className={rowDataClass}>
          <input
            id="isNullable"
            aria-describedby="comments-description"
            name="isNullable"
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
            checked={!column.isNullable}
            onChange={(event) => {
              dispatch({
                type: "UpdateColumn",
                columnIndex: columnIndex,
                column: {
                  ...column,
                  isNullable: !event.target.checked,
                },
              });
            }}
            disabled={columnReadOnly ? true : false}
          />
        </td>
        {columnReadOnly ? null : (
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
        )}
      </tr>
    );
  };

  return (
    <div>
      <table>
        <thead>
          {columns.length > 0 ? (
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
                Not Null
              </th>
              <th scope="col" className={headerClass}></th>
            </tr>
          ) : null}
        </thead>
        <tbody>
          {columns.map((c, index) => {
            return renderColumn(c, index);
          })}
        </tbody>
      </table>
      <button
        type="button"
        className="mx-auto mt-8 block w-80 rounded-md bg-gray-600 px-16 py-1.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
        onClick={() => {
          dispatch({
            type: "AddColumn",
            column: {
              name: null,
              fieldType: null,
              defaultValue: null,
              isNullable: true,
              extra: null,
            },
          });
        }}
      >
        Add Column
      </button>
    </div>
  );
}
