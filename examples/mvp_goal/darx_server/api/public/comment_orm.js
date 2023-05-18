export async function createComment(context, content, post_id) {
  const { auth, db } = context;
  if (!auth.uid) {
    return new Response().status(403).json({ error: "not authorized" });
  }

  return await db.txn(async (txn) => {
    const post = await txn.table("posts").findOne({
      where: {
        post_id: post_id,
        status: "published",
      },
    });

    if (!post) {
      return new Response().status(400).json({ error: "post not found" });
    }

    await txn.table("comments").insert({
      user_id: auth.uid,
      post_id: post_id,
      content: content,
    });

    await txn.table("posts").updateOne({
      where: {
        post_id: post_id,
      },
      data: {
        comments_count: {
          inc: 1,
        },
      },
    });

    return new Reponse().status(200);
  });
}
