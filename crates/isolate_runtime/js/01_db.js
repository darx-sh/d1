const core = globalThis.Deno.core;
const ops = core.ops;

// /**
//  * @param {number} id
//  */
// class Cursor {
//   constructor(rid) {
//     this.rid = rid;
//   }
//
//   next() {
//     return core.opAsync("op_db_cursor_next", this.rid);
//   }
// }

// class DBQuery {
//   /**
//    * @param {string} queryString
//    * @param {*[]} params
//    */
//   constructor(queryString, params) {
//     this.queryString = queryString;
//     this.params = params;
//   }
//
//   fetchAll() {
//     return core.opAsync("op_db_fetch_all", this.queryString, this.params);
//   }
//
//   exec() {
//     return core.opAsync("op_db_exec", this.queryString, this.params);
//   }
//
//   async cursor() {
//     return new Cursor(
//       await core.opAsync("op_db_cursor", this.queryString, this.params)
//     );
//   }
// }
/**
 * @param {string} queryString
 * @param {...*} params
 * @returns {DBQuery}
 */
function query(queryString, ...params) {
  return core.opAsync("op_db_query", queryString, params);
}

const db = {
  query: query,
};

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
