function isLoggedInUser(context, _) {
  return context.user != null;
}

function isPostOwner(context, post) {
  return isLoggedInUser(context, post) && context.user.id === post.user_id;
}

function postFilter(context, post) {
  if (post.status === "published") {
    return post;
  } else {
    return null;
  }
}

function isCommentOwner(context, comment) {
  return (
    isLoggedInUser(context, comment) && context.user.id === comment.user_id
  );
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
