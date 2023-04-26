export async function handler() {
  return await db.query('SELECT * FROM test LIMIT 10');
}
console.log('bar loaded')