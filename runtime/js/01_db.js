console.log("db.js extension loaded");

const core = globalThis.Deno.core;
const ops = core.ops;

class DBQuery {
  /**
   * @param {string} queryString
   * @param {*[]} params
   */
  constructor(queryString, params) {
    this.queryString = queryString;
    this.params = params;
  }

  fetchAll() {
    return core.opAsync("op_db_fetch_all", this.queryString, this.params);
  }
}
/**
 * @param {string} queryString
 * @param {...*} params
 * @returns {DBQuery}
 */
function query(queryString, ...params) {
  return new DBQuery(queryString, params);
}

const db = {
  query: query,
};

globalThis.db = db;
