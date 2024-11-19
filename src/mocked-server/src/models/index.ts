export interface Reply {
    id: string;
    user: UserPost;
    replyToId: string;
    content: string;
    nestedReplies: Reply[];
    likes: number;
    views: number;
    createdAt: Date;
    userInteracted: boolean;
}

export interface AuthenticationResult {
    success: boolean;
    id?: string;
    accessToken?: AccessToken;
    error?: string;
}

export interface AccessToken {
    value: string;
    expirationDate: Date;
}

export interface FileMetadata {
    fileName: string;
    contentType: string;
    url: string;
}

export interface PaginatedResponse<Type> {
    data: Type[];
    hasNextPage: boolean;
    perPage: number;
    next: string;
}

export interface UserSummary {
    id: string;
    username: string;
    createdAt: string;
}

export interface UserRelationship {
    followers: UserSummary[] | null;
    followedPeople: UserSummary[] | null;
    blockedPeople: UserSummary[] | null;
    friends: UserSummary[] | null;
    sentFriendRequests: UserSummary[] | null;
    receivedFriendRequests: UserSummary[] | null;
}

export interface UserPost {
    id: string;
    profileImage: FileMetadata;
    name: string;
    username: string;
    bio: string;
    location: string;
}

export interface Post {
    id: string;
    text: string;
    likes: number;
    views: number;
    user: UserPost;
    filesMetadata: FileMetadata[] | null;
    replies: Reply[] | null;
    createdAt: string;
    updatedAt: string;
    userInteracted: boolean;
}

export interface User {
    id: string;
    username: string;
    profileImage: FileMetadata;
    backgroundImage: FileMetadata;
    name: string;
    bio: string;
    location: string;
    birthDate: Date;
    relationships: UserRelationship;
    posts?: PaginatedResponse<Post>; // Make posts optional and paginated
    createdAt: Date;
}
