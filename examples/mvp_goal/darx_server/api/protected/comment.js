export async function createComment(context, content, user_id, post_id) {
  const { db } = context;

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
    return new Response().status(200);
  });
}
