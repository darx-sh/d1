const core = globalThis.Deno.core;
const ops = core.ops;

class DBConn {
  constructor(rid) {
    this.rid = rid;
  }

  execute(query, ...params) {
    return core.opAsync("op_db_execute", this.rid, query, params);
  }
  ddl(req) {
    return core.opAsync("op_ddl", this.rid, req);
  }
}

class SelectStatement {
  constructor(rid) {
    this.rid = rid;
  }

  columns(...fields) {
    ops.op_select_columns(this.rid, fields);
    return this;
  }

  from(tableName) {
    ops.op_select_from(this.rid, tableName);
    return this;
  }

  build() {
    return ops.op_select_build(this.rid);
  }

  async execute(conn) {
    const { sql, values } = this.build();
    return conn.execute(sql, values);
  }
}

function select() {
  const rid = core.ops.op_select_statement();
  return new SelectStatement(rid);
}

async function useDB() {
  // returns a db connection
  const rid = await core.opAsync("op_use_db");
  return new DBConn(rid);
}

let Dx = {};
Dx.env = new Proxy(
  {},
  {
    get(target, key) {
      const value = core.ops.op_var_get(`${key}`);
      if (value === null) {
        return undefined;
      } else {
        return value;
      }
    },
  }
);

globalThis.useDB = useDB;
globalThis.select = select;
globalThis.Dx = Dx;
