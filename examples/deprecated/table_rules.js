function isLoggedInUser({ auth }, _) {
  return auth.uid != null;
}

function isPostOwner(context, post) {
  const { auth } = context;
  return isLoggedInUser(context, post) && auth.uid === post.user_id;
}

function postFilter(context, post) {
  if (post.status === "published") {
    return post;
  } else {
    return null;
  }
}

function isCommentOwner(context, comment) {
  const { auth } = context;
  return isLoggedInUser(context, comment) && auth.uid === comment.user_id;
}

export const rules = {
  posts: {
    user_role: {
      create: isLoggedInUser,
      get: postFilter,
      update: isPostOwner,
      delete: isPostOwner,
    },
    service_role: {
      allow_all: true,
    },
  },
  comments: {
    user_role: {
      create: isLoggedInUser,
      get: (context, comment) => comment,
      update: isCommentOwner,
      delete: isCommentOwner,
    },
    service_role: {
      allow_all: true,
    },
  },
};