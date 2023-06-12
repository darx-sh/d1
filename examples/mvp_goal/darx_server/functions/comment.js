export async function createComment(context, content, post_id) {
  const db = await useDB();
  const { auth } = context;
  if (!auth.uid()) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  return await db.txn(async (txn) => {
    const posts = await txn.exec("SELECT * FROM posts WHERE id = ? LIMIT 1", [
      post_id,
    ]);

    if (posts[0].status !== "published") {
      return new Response().status(403).json({ error: "not authorized" });
    }

    await txn.exec(
      "INSERT INTO comments WHERE content = ? AND user_id = ? AND post_id = ?",
      [content, auth.user_id, post_id]
    );

    await txn.exec(
      "UPDATE posts SET comments_count = comments_count + 1 WHERE id = ?",
      [post_id]
    );
    return new Response().status(200);
  });
}
