const r1 = await db.query("TRUNCATE TABLE test");
console.log("result: ", r1);

for (let i = 0; i < 100; i++) {
  const result = await db.query("INSERT INTO test (name) VALUES (?)", "foo");
  // console.log("result: ", result);
}

var r2 = await db.query("SELECT COUNT(*) FROM test");
console.log("result: ", r2);

let r3 = await db.query("SELECT * from test WHERE name = ?", "foo");
console.log("result: ", r3);
