import { DataGridPro, GridColDef, GridRowsProp } from "@mui/x-data-grid-pro";
import {
  useDatabaseDispatch,
  Row,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import {
  Datum,
  displayDatum,
  PrimitiveType,
  primitiveToDatum,
} from "~/utils/types";

import {
  RowDatumToApiRow,
  ApiRow,
  ApiRowToRowDatum,
} from "~/components/project/database/Api";
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
        const value = p.value as PrimitiveType;
        const datum = primitiveToDatum(value, c.fieldType);
        if (datum.typ === "NULL") {
          return (
            <div key={c.name}>
              <span className="rounded-md bg-gray-200 p-1 text-gray-500">
                NULL
              </span>
            </div>
          );
        }
        return <div key={c.name}>{displayDatum(datum)}</div>;
      },
    };
  });

  gridColDef.push({
    field: "actions",
    type: "actions",
    getActions: (params) => [
      <GridActionsCellItem
        key="update"
        icon={<EditIcon />}
        label="Edit"
        onClick={() => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-assignment
          const apiRow = params.row as ApiRow;
          const row = ApiRowToRowDatum(apiRow, tableDef);
          dispatch({
            type: "InitRowEditorFromRow",
            row,
          });
        }}
      />,
      <GridActionsCellItem key="delte" icon={<DeleteIcon />} label="Delete" />,
    ],
  });

  const gridRows = rows.map((r) => {
    return RowDatumToApiRow(r);
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
        initialState={{ pinnedColumns: { right: ["actions"] } }}
      ></DataGridPro>
    </div>
  );
}
