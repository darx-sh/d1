const db = await useDB();
const r1 = await db.execute("DROP TABLE IF EXISTS test");
console.log(r1);

const r2 = await db.ddl({
  createTable: {
    tableName: "test",
    columns: [
      {
        name: "name",
        fieldType: "text",
        isNullable: false,
      },
    ],
  },
});
console.log(r2);

const r3 = await db.ddl({
  addColumn: {
    tableName: "test",
    column: {
      name: "age",
      fieldType: "int64",
      isNullable: false,
      defaultValue: { int64: 0 },
    },
  },
});
console.log(r3);

const r4 = await db.ddl({
  dropColumn: {
    tableName: "test",
    columnName: "age",
  },
});
console.log(r4);
