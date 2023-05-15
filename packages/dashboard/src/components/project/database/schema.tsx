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
    "rounded-md bg-indigo-600 px-2.5 py-1.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600";

  return (
    <div>
      <button type="button" className={btnClass}>
        Sync Schema
      </button>
      <JsEditor initialCode={code} />
    </div>
  );
}
