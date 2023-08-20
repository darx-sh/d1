export async function listTable() {
  const db = await useDB();
  return db.execute("SELECT 1");
}
