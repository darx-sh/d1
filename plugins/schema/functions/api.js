export async function ddl(req) {
  const db = await useDB();
  return await db.ddl(req);
}

export async function listTable() {
  const db = await useDB();
  const { rows } = await db.execute(
    `\
  SELECT \
    TABLE_NAME AS tableName, \
    COLUMN_NAME AS columnName, \
    DATA_TYPE AS columnType, \
    IS_NULLABLE AS nullable, \
    COLUMN_DEFAULT AS defaultValue, \
    COLUMN_COMMENT AS comment \
  FROM \
    INFORMATION_SCHEMA.COLUMNS \
  WHERE \
      TABLE_SCHEMA = ? \
  ORDER BY \
    TABLE_NAME ASC, \
    ORDINAL_POSITION ASC
    `,
    Darx.env.DX_DB_NAME
  );
  return rows;
}

export async function listTableNames() {
  const db = await useDB();
  const { rows } = await db.execute(
    "SELECT TABLE_NAME AS tableName FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = ?",
    Darx.env.DX_DB_NAME
  );
  return rows;
}
