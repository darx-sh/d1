const db = await useDB();

const r1 = await db.execute("TRUNCATE TABLE test");
console.log("truncate result: ", r1);

for (let i = 0; i < 10; i++) {
  const result = await db.execute("INSERT INTO test (name) VALUES (?)", "foo");
  console.log("insert result: ", result);
}

var r2 = await db.execute("SELECT COUNT(*) AS count FROM test");
console.log("count result: ", r2);

let r3 = await db.execute("SELECT * from test WHERE name = ?", "foo");
console.log("select result: ", r3);

// select().columns("id", "name").from("test").build();
