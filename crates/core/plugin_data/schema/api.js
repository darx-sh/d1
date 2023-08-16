export async function ddl(req) {
  const db = useDB();
  return await db.ddl(req);
}

export async function listTable() {
  const db = useDB();
  // todo: replace dx_8nvcym53y8d2 with env var
  return await db.execute(
    "SELECT\n" +
      "  TABLE_NAME AS `table`,\n" +
      "  COLUMN_NAME AS `column`,\n" +
      "  DATA_TYPE AS `type`,\n" +
      "  IS_NULLABLE AS `nullable`,\n" +
      "  COLUMN_DEFAULT AS `default`,\n" +
      "  COLUMN_COMMENT AS `comment`\n" +
      "FROM\n" +
      "  INFORMATION_SCHEMA.COLUMNS\n" +
      "WHERE\n" +
      "    TABLE_SCHEMA = 'dx_8nvcym53y8d2'\n" +
      "ORDER BY\n" +
      "  TABLE_NAME ASC,\n" +
      "  ORDINAL_POSITION ASC;"
  );
}
