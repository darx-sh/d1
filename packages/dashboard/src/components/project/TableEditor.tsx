import ColumnTypeSelect from "~/components/project/ColumnTypeSelect";

export default function TableEditor() {
  const headerClass = "px-2 text-sm font-light italic text-left";
  const rowClass = "px-2 py-2 text-sm font-normal";
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
          </tr>
        </thead>
        <tbody>
          <tr>
            <td className={rowClass}>
              <input
                type="text"
                name="id"
                id="id"
                className="block w-28 rounded-md border-0 py-1.5 pl-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
                placeholder="id"
              />
            </td>
            <td className={rowClass}>
              <ColumnTypeSelect></ColumnTypeSelect>
            </td>
            <td className={rowClass}>
              <input
                type="text"
                name="id"
                id="email"
                className="block w-28 rounded-md border-0 py-1.5 pl-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600"
                placeholder="NULL"
              />
            </td>
            <td className={rowClass}>
              <input
                id="primary"
                aria-describedby="comments-description"
                name="primary"
                type="checkbox"
                className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
              />
            </td>
            <td className={rowClass}>
              <input
                id="isNullable"
                aria-describedby="comments-description"
                name="isNullable"
                type="checkbox"
                className="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600"
              />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}
