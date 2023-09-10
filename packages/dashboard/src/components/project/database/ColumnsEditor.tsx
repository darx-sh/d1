import { ArchiveBoxXMarkIcon, Bars3Icon } from "@heroicons/react/24/outline";
import ColumnTypeSelect from "~/components/project/database/ColumnTypeSelect";
import {
  DxColumnType,
  useDatabaseState,
  useDatabaseDispatch,
} from "~/components/project/database/DatabaseContext";
import {
  displayDefaultValue,
  FieldType,
  stringToDefaultValue,
} from "~/utils/types";

import classNames from "classnames";

export default function ColumnsEditor() {
  const headerClass = "px-2 text-sm font-light italic text-center";
  const rowDataClass = "px-2 py-2 text-sm font-normal text-center";
  const dispatch = useDatabaseDispatch();
  const state = useDatabaseState();

  const columns = state.draftTable.columns;
  const columnMarks = state.draftColumnMarks;

  const columnIsReadOnly = (columnIndex: number) => {
    return ["id", "created_at", "updated_at"].includes(
      columns[columnIndex]!.name
    );
  };

  const canChangeColumnProperty = (columnIndex: number) => {
    // cannot change existing column's "type", "not null" and "default value".
    return columnMarks[columnIndex] === "Add";
  };

  const renderNotNull = (
    column: DxColumnType,
    columnIndex: number,
    isReadOnly: boolean
  ) => {
    return (
      <td
        className={classNames(
          column.fieldType === null ? "pointer-events-none invisible" : "",
          rowDataClass
        )}
      >
        <input
          id="isNullable"
          aria-describedby="comments-description"
          name="isNullable"
          type="checkbox"
          className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gray-600"
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
          disabled={
            isReadOnly || !canChangeColumnProperty(columnIndex) ? true : false
          }
        />
      </td>
    );
  };

  const renderDefaultValue = (
    column: DxColumnType,
    columnIndex: number,
    isReadOnly: boolean
  ) => {
    return (
      <td
        className={classNames(
          column.fieldType === null ? "pointer-events-none invisible" : "",
          rowDataClass
        )}
      >
        <input
          type="text"
          name="defaultValue"
          id="defaultValue"
          className="block w-44 rounded-md border-0 py-1.5 pl-2 text-xs text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gray-300"
          placeholder={displayDefaultValue(column.defaultValue)}
          onChange={(event) => {
            dispatch({
              type: "UpdateColumn",
              columnIndex,
              column: {
                ...column,
                defaultValue: stringToDefaultValue(
                  column.fieldType,
                  event.target.value
                ),
              },
            });
          }}
          disabled={
            isReadOnly ||
            column.fieldType === null ||
            !canChangeColumnProperty(columnIndex)
              ? true
              : false
          }
        />
      </td>
    );
  };

  const renderDelete = (
    column: DxColumnType,
    columnIndex: number,
    isReadOnly: boolean
  ) => {
    return isReadOnly ? null : (
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
    );
  };
  const renderColumn = (column: DxColumnType, columnIndex: number) => {
    const mark = columnMarks[columnIndex];
    if (mark === "Del" || mark === "None") {
      return null;
    }

    const columnName = column.name ?? "";
    const columnReadOnly = columnIsReadOnly(columnIndex);

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
              "block w-32 rounded-md border-0 py-1.5 pl-2 text-sm text-gray-400 shadow-sm ring-1 ring-inset ring-gray-300 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gray-300"
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
            onSelect={(t: FieldType) => {
              dispatch({
                type: "UpdateColumn",
                columnIndex,
                column: {
                  ...column,
                  fieldType: t,
                },
              });
            }}
            disabled={columnReadOnly || !canChangeColumnProperty(columnIndex)}
          ></ColumnTypeSelect>
        </td>

        {renderNotNull(column, columnIndex, columnReadOnly)}
        {renderDefaultValue(column, columnIndex, columnReadOnly)}
        {renderDelete(column, columnIndex, columnReadOnly)}
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
                Not Null
              </th>
              <th scope="col" className={headerClass}>
                Default Value
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
              name: "",
              fieldType: "NotDefined",
              defaultValue: { typ: "NotDefined", value: "" },
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
