const BaseAPIUri = "http://localhost:10000"

export const SignUpUri = `${BaseAPIUri}/auth/register`
export const SignInUri = `${BaseAPIUri}/auth/login`
export const VerifyEmailByOtpCode = (id: string): string => `${BaseAPIUri}/auth/users/${id}/verify/email`
export const RefreshTokenUri = `${BaseAPIUri}/auth/token/refresh`
export const RevokeTokenUri = `${BaseAPIUri}/auth/token/revoke`

export const GetUserUri = (id: string, currentUserId: string | null): string =>
    currentUserId
        ? `${BaseAPIUri}/a/users/${id}?interactionUserId=${currentUserId}`
        : `${BaseAPIUri}/a/users/${id}`
export const UpdateUserUri = (id: string): string => `${BaseAPIUri}/users/${id}`
export const DeleteUserUri = (id: string): string => `${BaseAPIUri}/users/${id}`

export const GetFollowersUri = (id: string): string => `${BaseAPIUri}/users/${id}/followers`
export const GetFollowedUri = (id: string): string => `${BaseAPIUri}/users/${id}/followers/me`
export const FollowUserUri = (currentUserId: string, followedUserId: string): string => `${BaseAPIUri}/users/${currentUserId}/followers/${followedUserId}`
export const UnfollowUserUri = (currentUserId: string, followedUserId: string): string => `${BaseAPIUri}/users/${currentUserId}/followers/${followedUserId}`

export const GetFriendsUri = (id: string): string => `${BaseAPIUri}/users/${id}/friends`
export const RemoveFriendUri = (id: string, friendID: string): string => `${BaseAPIUri}/users/${id}/friends/${friendID}`
export const GetPendingRequestsUri = (id: string): string => `${BaseAPIUri}/users/${id}/friends/requests`
export const SendFriendRequestUri = (currentUserId: string, receiverId: string): string => `${BaseAPIUri}/users/${currentUserId}/friends/add/${receiverId}`
export const AcceptFriendRequestUri = (currentUserId: string, receiverId: string): string => `${BaseAPIUri}/users/${currentUserId}/friends/accept/${receiverId}`
export const DeclineFriendRequestUri = (currentUserId: string, receiverId: string): string => `${BaseAPIUri}/users/${currentUserId}/friends/decline/${receiverId}`

export const GetBlockedUsersUri = (id: string): string => `${BaseAPIUri}/users/${id}/blocks`
export const BlockUserUri = (id: string, userToBlockId: string): string => `${BaseAPIUri}/users/${id}/blocks/${userToBlockId}`
export const UnblockUserUri = (id: string, blockedUserId: string): string => `${BaseAPIUri}/users/${id}/blocks/${blockedUserId}`

export const GetUsersFeed = (userId: string, next?: string | null, perPage?: string | null): string =>
    next ?
        `${BaseAPIUri}/a/users/${userId}/feed?pageSize=${perPage ?? 10}&next=${next}`
        : `${BaseAPIUri}/a/users/${userId}/feed`

export const GetUsersPostsUri = (userId: string, next?: string | null, perPage?: string | null): string =>
    next ?
        `${BaseAPIUri}/users/${userId}/posts?perPage=${perPage ?? 10}&next=${next}`
        : `${BaseAPIUri}/a/users/${userId}/posts`

export const GetPostUri = (id: string, userId: string): string => `${BaseAPIUri}/a/posts/${id}?userInteractionId=${userId}`
export const GetReplyUri = (id: string, postId: string, userId: string): string => `${BaseAPIUri}/posts/${postId}/replies/${id}?userInteractionId=${userId}`
export const CreatePostUri = (userId: string): string => `${BaseAPIUri}/users/${userId}/posts`
export const UpdatePostUri = (userId: string, postId: string): string => `${BaseAPIUri}/users/${userId}/posts/${postId}`
export const DeletePostUri = (userId: string, postId: string): string => `${BaseAPIUri}/users/${userId}/posts/${postId}`

export const LikePostUri = (userId: string, postId: string): string => `${BaseAPIUri}/users/${userId}/likes/posts/${postId}`
export const UnlikePostUri = (userId: string, postId: string): string => `${BaseAPIUri}/users/${userId}/likes/posts/${postId}`
export const ViewPostUri = (userId: string, postId: string): string => `${BaseAPIUri}/users/${userId}/views/posts/${postId}`

export const CreateReplyUri = (postId: string): string => `${BaseAPIUri}/posts/${postId}/replies`
export const UpdateReplyContentUri = (postId: string, replyId: string): string => `${BaseAPIUri}/posts/${postId}/replies/${replyId}`
export const DeleteReplyUri = (postId: string, replyId: string): string => `${BaseAPIUri}/posts/${postId}/replies/${replyId}`
