import JsEditor from "~/components/project/editor";

export default function Schema() {
  const code = `table "orders" {
  schema = schema.example
  column "id" {
    null = false
    type = int
  }
  column "user_id" {
    null = true
    type = varchar(100)
  }
  primary_key {
    columns = [column.id]
  }
}`;
  const btnClass =
    "mr-5 rounded-md bg-indigo-600 px-2.5 py-1.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600";

  return (
    <div>
      <div>Pending Migrations...</div>
      <button type="button" className={btnClass}>
        Approve Migration
      </button>
      <button type="button" className={btnClass}>
        Apply Migration
      </button>
      <div className="mt-2">
        <JsEditor initialCode={code} readOnly={true} />
      </div>
    </div>
  );
}
