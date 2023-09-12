import {
  DataGridPro,
  GridColDef,
  GridRenderEditCellParams,
  GridRowsProp,
} from "@mui/x-data-grid-pro";
import {
  DxColumnType,
  isSystemField,
  Row,
  TableDef,
} from "~/components/project/database/DatabaseContext";
import { displayColumnValue } from "~/utils/types";
import { GridActionsCellItem, GridRowId } from "@mui/x-data-grid-pro";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import { DateTimePicker } from "@mui/x-date-pickers/DateTimePicker";
import dayjs from "dayjs";

interface TableGridProp {
  tableDef: TableDef;
  rows: Row[];
}

export default function TableGrid(prop: TableGridProp) {
  const { tableDef, rows } = prop;
  const columns = tableDef.columns;

  const getFieldType = (column: DxColumnType) => {
    switch (column.fieldType) {
      case "int64":
      case "int64Identity":
        return "number";
      case "float64":
        return "number";
      case "varchar(255)":
      case "text":
        return "string";
      case "datetime":
        return "string";
      case "bool":
        return "boolean";
    }
  };

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
      <GridActionsCellItem key="update" icon={<EditIcon />} label="Edit" />,
      <GridActionsCellItem key="delte" icon={<DeleteIcon />} label="Delete" />,
    ],
  });

  const gridRows: GridRowsProp = rows.map((r) => {
    return r;
  });

  return (
    <div className="mt-3">
      <DataGridPro
        autoHeight
        disableColumnFilter
        paginationMode="server"
        columns={gridColDef}
        rows={gridRows}
      ></DataGridPro>
    </div>
  );
}
