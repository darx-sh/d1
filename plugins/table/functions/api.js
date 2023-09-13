export async function paginateTable(tableName, prevCreatedAt, prevIds, limit) {
  const whereCreated = (created_at) => {
    if (created_at) {
      return `WHERE created_at >= ?`;
    } else {
      return ``;
    }
  };

  const whereIds = (ids) => {
    if (ids) {
      return ` AND id NOT IN (${ids.map(() => "?").join(", ")})`;
    } else {
      return ``;
    }
  };

  const paramCreated = (created_at) => {
    if (created_at) {
      return [created_at];
    } else {
      return [];
    }
  };

  const paramIds = (ids) => {
    if (ids) {
      return ids;
    } else {
      return [];
    }
  };

  const whereFragment = whereCreated(prevCreatedAt) + whereIds(prevIds);
  const sql = `SELECT * FROM ${tableName} ${whereFragment} ORDER BY created_at ASC LIMIT ?`;
  const params = [...paramCreated(prevCreatedAt), ...paramIds(prevIds), limit];
  const db = await useDB();
  const { rows } = await db.execute(sql, ...params);
  return rows;
}

export async function insertRow(tableName, values) {
  const columnNames = Object.keys(values);
  const colNamesFragment = columnNames.join(", ");
  const columnValues = Object.values(values);
  const valuesPlaceholder = columnValues.map(() => "?").join(", ");
  const sql = `INSERT INTO ${tableName} (${colNamesFragment}) VALUES (${valuesPlaceholder})`;
  const db = await useDB();
  return await db.execute(sql, ...columnValues);
}

export async function updateRow(tableName, id, values) {
  const columnNames = Object.keys(values);
  const columnValues = Object.values(values);
  const setFragment = columnNames.map((name) => `${name} = ?`).join(", ");
  const sql = `UPDATE ${tableName} SET ${setFragment} WHERE id = ?`;
  const params = [...columnValues, id];
  const db = await useDB();
  return await db.execute(sql, ...params);
}

export async function deleteRows(tableName, ids) {
  const placeholders = ids.map(() => "?").join(", ");
  const sql = `DELETE FROM ${tableName} WHERE id IN (${placeholders})`;
  const params = ids;
  const db = await useDB();
  return await db.execute(sql, ...params);
}
