import { useDB } from "darx";

export async function viewPublishedPost(context, post_id) {
  const db = useDB();
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

export async function createPost(context, content) {
  const db = useDB();
  const { auth } = context;

  if (!auth) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  await db.exec("INSERT INTO posts WHERE content = ?, user_id = ?", [
    content,
    auth.uid,
  ]);
  return new Response().status(201);
}

export async function publishPost(context, post_id) {
  const db = useDB();
  const { auth } = context;

  if (!auth) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  const result = await db.exec(
    "UPDATE posts SET status = ? WHERE id = ? AND user_id = ? AND status = ?",
    ["published", post_id, auth.user_id, "draft"]
  );

  return new Response().status(200).json({ result: result });
}
