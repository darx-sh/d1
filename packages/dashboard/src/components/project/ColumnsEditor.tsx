import { ArchiveBoxXMarkIcon, Bars3Icon } from "@heroicons/react/24/outline";
import ColumnTypeSelect from "~/components/project/ColumnTypeSelect";
import {
  ColumnDef,
  displayDefaultValue,
} from "~/components/project/DatabaseContext";

interface ColumnsEditorProps {
  columns: ColumnDef[];
}

export default function ColumnsEditor(props: ColumnsEditorProps) {
  const headerClass = "px-2 text-sm font-light italic text-center";
  const rowDataClass = "px-2 py-2 text-sm font-normal text-center";
  const { columns } = props;

  const renderColumn = (column: ColumnDef) => {
    return (
      <tr className="hover:bg-gray-200">
        <td className={rowDataClass}>
          <input
            type="text"
            name={column.name}
            id={column.name}
            className="block w-32 rounded-md border-0 py-1.5 pl-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
            placeholder={column.name}
            defaultValue={column.name}
          />
        </td>
        <td className={rowDataClass}>
          <ColumnTypeSelect></ColumnTypeSelect>
        </td>
        <td className={rowDataClass}>
          <input
            type="text"
            name="defaultValue"
            id="defaultValue"
            className="block w-28 rounded-md border-0 py-1.5 pl-2 text-xs text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
            placeholder={displayDefaultValue(column.defaultValue)}
            value={displayDefaultValue(column.defaultValue)}
          />
        </td>
        <td className={rowDataClass}>
          <input
            id="primary"
            aria-describedby="comments-description"
            name="primary"
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
          />
        </td>
        <td className={rowDataClass}>
          <input
            id="isNullable"
            aria-describedby="comments-description"
            name="isNullable"
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
          />
        </td>
        <td className={rowDataClass}>
          <ArchiveBoxXMarkIcon className="h-6 w-6" aria-hidden="true" />
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
          {columns.map((c) => {
            return renderColumn(c);
          })}
        </tbody>
      </table>
    </div>
  );
}
