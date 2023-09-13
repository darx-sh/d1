import {
  DataGridPro,
  GridColDef,
  GridRowsProp,
} from "@mui/x-data-grid-pro";
import {
  useDatabaseDispatch,
  Row,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import { displayColumnValue } from "~/utils/types";
import { GridActionsCellItem } from "@mui/x-data-grid-pro";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";

interface TableGridProp {
  tableDef: TableDef;
  rows: Row[];
}

export default function TableGrid(prop: TableGridProp) {
  const { tableDef, rows } = prop;
  const columns = tableDef.columns;
  const dispatch = useDatabaseDispatch();

  const gridColDef: GridColDef[] = columns.map((c) => {
    return {
      field: c.name,
      sortable: false,
      editable: false,

      renderCell: (p) => {
        if (p.value === null) {
          return (
            <div key={c.name}>
              <span className="rounded-md bg-gray-200 p-1 text-gray-500">
                NULL
              </span>
            </div>
          );
        }

        if (c.fieldType === "datetime") {
          const d = new Date(p.value as string);
          return d.toLocaleString();
        }

        return (
          <div key={c.name}>{displayColumnValue(p.value, c.fieldType)}</div>
        );
      },
    };
  });

  gridColDef.push({
    field: "actions",
    type: "actions",
    getActions: (params) => [
      <GridActionsCellItem key="update" icon={<EditIcon />} label="Edit" onClick={() => {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-assignment
        dispatch({type: "InitRowEditorFromRow", row: params.row.__originalRow})
      }}/>,
      <GridActionsCellItem key="delte" icon={<DeleteIcon />} label="Delete"/>,
    ],
  });

  const gridRows: GridRowsProp = rows.map((r) => {
    return {__originalRow: r, ...r};
  });

  return (
    <div className="mt-3">
      <DataGridPro
        autoHeight
        disableColumnFilter
        paginationMode="server"
        rowCount={100}
        columns={gridColDef}
        rows={gridRows}
      ></DataGridPro>
    </div>
  );
}
