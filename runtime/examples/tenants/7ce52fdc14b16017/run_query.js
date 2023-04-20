let r1 = await db.query("SELECT * from test WHERE name = ?", "foo");
console.log("r1: ", r1);

let r2 = await db.query("INSERT INTO test (name) VALUES (?)", "foo");
console.log("r2: ", r2);

let r3 = await db.query("SELECT * from test");
console.log("r3: ", r3);

let r4 = await db.query("SELECT * from test WHERE name = ?", "barbar");
console.log("r4: ", r4);

let r5 = await db.query(
  "UPDATE test set name = ? WHERE id > ?",
  "foo",
  23232322
);
console.log("r5: ", r5);
