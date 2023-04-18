let result = await db.query("SELECT * from test").fetchAll();
console.log("fetch result: ", result);
