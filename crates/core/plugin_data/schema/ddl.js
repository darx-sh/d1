export default async function ddl(req) {
  const db = useDB();
  return await db.ddl(req);
}
