export async function viewPublishedPost(context, post_id) {
  const { db } = await useDB();
  const post = await db.table("posts").findOne({
    id: post_id,
    status: "published",
  });
  return post;
}

export async function createPost(context, content) {
  const db = await useDB();
  const { auth } = context;

  if (!auth.uid) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  await db.table("posts").insert({
    content: content,
    user_id: auth.uid,
  });
  return new Response().status(201);
}

export async function publishPost(context, post_id) {
  const db = await useDB();
  const { auth } = context;

  if (!auth) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  const result = await db.table("posts").updateFirst({
    where: {
      user_id: auth.uid,
      status: "draft",
    },
    data: {
      status: "published",
    },
  });

  return new Response().status(200).json({ result: result });
}
