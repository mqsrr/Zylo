
export interface UserRelationship {
    follows: UserFollowRequests;
    blocks: UserRelationshipData;
    friends: UserRelationshipData;
    friendRequests: UserFriendRequests;
}

interface UserFollowRequests {
    followers: UserRelationshipData,
    following: UserRelationshipData,
}

interface UserFriendRequests {
    sent: UserRelationshipData,
    received: UserRelationshipData,
}


interface UserRelationshipData {
    ids: string[],
    created_at: Map<String, String>,
}
