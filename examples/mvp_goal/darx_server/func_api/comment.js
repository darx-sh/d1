export async function createComment({ db }, user_id, content, post_id) {
  const user = await db.exec("SELECT * FROM users WHERE id = ? LIMIT 1", [
    user_id,
  ]);

  if (!user || user.length === 0) {
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
      "INSERT INTO comments WHERE content = ?, user_id = ?, post_id = ?",
      [content, user_id, post_id]
    );

    await txn.exec(
      "UPDATE posts SET comments_count = comments_count + 1 WHERE id = ?",
      [post_id]
    );
  });
}
