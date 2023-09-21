export async function ddl(req) {
  const db = await useDB();
  return await db.ddl(req);
}

export async function listTable() {
  const db = await useDB();
  let schema = [];

  // Get table names
  const { rows: rTableNames } = await db.execute(
    "SELECT TABLE_NAME AS tableName FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = ?",
    Darx.env.DX_DB_NAME
  );

  for (const { tableName } of rTableNames) {
    // Get column names
    const { rows: rColumns } = await db.execute(
      `SELECT \
    COLUMN_NAME AS columnName, \
    DATA_TYPE AS fieldType, \
    IS_NULLABLE AS nullable, \
    COLUMN_DEFAULT AS defaultValue, \
    COLUMN_COMMENT AS comment, \
    EXTRA as extra \
  FROM \
    INFORMATION_SCHEMA.COLUMNS \
  WHERE \
      TABLE_SCHEMA = ? AND TABLE_NAME = ?\
  ORDER BY ORDINAL_POSITION ASC`,
      Darx.env.DX_DB_NAME,
      tableName
    );

    // Get primary key
    const { rows: rPrimaryKey } = await db.execute(
      `\
SELECT COLUMN_NAME as columnName \
FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE \
WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? AND CONSTRAINT_NAME = 'PRIMARY'
    `,
      Darx.env.DX_DB_NAME,
      tableName
    );
    const tableDef = {
      tableName,
      columns: rColumns,
      primaryKey: rPrimaryKey.map((p) => {
        return p.columnName;
      }),
    };
    schema.push(tableDef);
  }

  return schema;
}

export async function listTableNames() {
  const db = await useDB();
  const { rows } = await db.execute(
    "SELECT TABLE_NAME AS tableName FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = ?",
    Darx.env.DX_DB_NAME
  );
  return rows;
}
