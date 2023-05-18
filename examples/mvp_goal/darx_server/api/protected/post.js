export async function viewPublishedPost(context, post_id) {
  const { db } = context;
  const posts = await db.exec(
    "SELECT * FROM posts WHERE id = ? AND status = ? LIMIT 1",
    [post_id, "published"]
  );
  if (posts.length > 0) {
    return posts[0];
  } else {
    return [];
  }
}

export async function createPost(context, user_id, content) {
  const { db } = context;

  await db.exec("INSERT INTO posts WHERE content = ?, user_id = ?", [
    content,
    user_id,
  ]);
  return new Response().status(201);
}

export async function publishPost(context, user_id, post_id) {
  const { db } = context;

  const result = await db.exec(
    "UPDATE posts SET status = ? WHERE id = ? AND user_id = ? AND status = ?",
    ["published", post_id, user_id, "draft"]
  );

  return new Response().status(200).json({ result: result });
}
