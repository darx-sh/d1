const db = await useDB();
const r1 = await db.execute("TRUNCATE TABLE test");
console.log("result: ", r1);

for (let i = 0; i < 100; i++) {
  const result = await db.execute("INSERT INTO test (name) VALUES (?)", "foo");
  console.log("result: ", result);
}

var r2 = await db.execute("SELECT COUNT(*) FROM test");
console.log("result: ", r2);

let r3 = await db.execute("SELECT * from test WHERE name = ?", "foo");
console.log("result: ", r3);
