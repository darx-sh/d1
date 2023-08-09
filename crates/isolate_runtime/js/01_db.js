const core = globalThis.Deno.core;
const ops = core.ops;

class DBConn {
  constructor(rid) {
    this.rid = rid;
  }

  execute(query, ...params) {
    return core.opAsync("op_db_execute", this.rid, query, params);
  }
}

async function useDB() {
  // returns a db connection
  const rid = await core.opAsync("op_use_db");
  return new DBConn(rid);
}

globalThis.useDB = useDB;
