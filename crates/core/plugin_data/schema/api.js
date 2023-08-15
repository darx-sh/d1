export default async function api(req) {
  const db = useDB();
  return await db.ddl(req);
}
