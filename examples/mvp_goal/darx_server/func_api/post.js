export async function createPost({ db }, user_id, content) {
  const user = await db.exec("SELECT * FROM users WHERE id = ? LIMIT 1", [
    user_id,
  ]);

  if (!user || user.length === 0) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  const data = await db.exec(
    "INSERT INTO posts WHERE content = ?, user_id = ?",
    [content, user_id]
  );
  return new Response().status(201);
}

export async function publishPost({ db }, user_id, post_id, content) {
  const user = await db.exec("SELECT * FROM users WHERE id = ? LIMIT 1", [
    user_id,
  ]);

  if (!user || user.length === 0) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  const posts = await db.exec("SELECT * FROM posts WHERE id = ? LIMIT 1", [
    post_id,
  ]);

  if (posts[0].user_id != user_id) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  await db.exec(
    "UPDATE posts SET content = ? status = 'published' WHERE id = ?",
    [content, post_id]
  );

  return new Response().status(200);
}
