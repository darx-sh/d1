function isOwner(context, record) {
  const { auth } = context;
  return auth.uid() === record.user_id;
}

export const rules = {
  posts: [
    {
      role: "authenticated",
      create: true,
      view: ({ auth }, post) => {
        // authenticated user can view any published post, or any post that the user owns.
        return auth.uid() === post.user_id || post.status === "published";
      },
      update: isOwner,
      delete: isOwner,
    },
    {
      role: "anon",
      // anno user can view any published post.
      get: (_, post) => {
        return post.status === "published";
      },
    },
  ],
  comments: [
    {
      role: "authenticated",
      // can only create post that is published.
      create: async ({ auth, db }, comment) => {
        const posts = await db.exec(
          "SELECT COUNT(*) as count FROM posts WHERE id = ? AND status = ?",
          [comment.post_id, "published"]
        );
        return posts.length > 0;
      },
      get: true,
      update: isOwner,
      delete: isOwner,
    },
    {
      role: "anon",
      get: true,
    },
  ],
};
