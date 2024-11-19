import { UserSummary } from "./User";

export interface UserRelationship {
    followers: UserSummary[] | null;
    followedPeople: UserSummary[] | null;
    blockedPeople: UserSummary[] | null;
    friends: UserSummary[] | null;
    sentFriendRequests: UserSummary[] | null;
    receivedFriendRequests: UserSummary[] | null;
}
