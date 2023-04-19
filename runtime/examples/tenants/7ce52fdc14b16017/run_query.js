let r1 = await db.query("SELECT * from test WHERE name = ?", "foo").fetchAll();
console.log("fetchAll: ", r1);

let r2 = await db.query("INSERT INTO test (name) VALUES (?)", "foo").exec();
console.log("exec: ", r2);
