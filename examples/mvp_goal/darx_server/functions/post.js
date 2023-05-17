export async function createPost({ user, db }, content) {
  if (!user) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  const data = await db.exec(
    "INSERT INTO posts WHERE content = ?, user_id = ?",
    [content, user.id]
  );
  return new Response().status(201);
}

export async function publishPost({ user, db }, post_id, content) {
  if (!user) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  const posts = await db.exec("SELECT * FROM posts WHERE id = ? LIMIT 1", [
    post_id,
  ]);

  if (posts[0].user_id != user.id) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  await db.exec(
    "UPDATE posts SET content = ? status = 'published' WHERE id = ?",
    [content, post_id]
  );

  return new Response().status(200);
}
