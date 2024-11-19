import {UserSummary} from "@/models/User.ts";
import {
    AcceptFriendRequestUri,
    BlockUserUri,
    DeclineFriendRequestUri,
    FollowUserUri,
    GetBlockedUsersUri,
    GetFollowedUri,
    GetFollowersUri,
    GetFriendsUri,
    GetPendingRequestsUri,
    RemoveFriendUri,
    SendFriendRequestUri,
    UnblockUserUri,
    UnfollowUserUri
} from "@/constants/requestsUri.ts";

class SocialConnectionsService {

    getFollowers = async (id: string, token: string): Promise<UserSummary[] | null> => {
        const response = await fetch(GetFollowersUri(id), {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
            }
        });

        return response.ok
            ? await response.json()
            : null;
    }

    getFollowed = async (id: string, token: string): Promise<UserSummary[] | null> => {
        const response = await fetch(GetFollowedUri(id), {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
            }
        });

        return response.ok
            ? await response.json()
            : null;
    }

    followUser = async (id: string, followedUserId: string, token: string): Promise<boolean> => {
        const response = await fetch(FollowUserUri(id, followedUserId), {
            method: "POST",
            headers: {
                Authorization: `Bearer ${token}`,
            }
        });

        return response.ok;
    }

    unfollowUser = async (id: string, followedUserId: string, token: string): Promise<boolean> => {
        const response = await fetch(UnfollowUserUri(id, followedUserId), {
            method: 'DELETE',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });


        return response.ok;
    }

    getFriends = async (id: string, token: string): Promise<UserSummary[] | null> => {
        const response = await fetch(GetFriendsUri(id), {
            method: 'GET',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
            ? await response.json()
            : null;
    }

    getPendingFriendRequests = async (id: string, token: string): Promise<UserSummary[] | null> => {
        const response = await fetch(GetPendingRequestsUri(id), {
            method: 'GET',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
            ? await response.json()
            : null;
    }

    removeFriend = async (id: string, friendID: string, token: string): Promise<boolean> => {
        const response = await fetch(RemoveFriendUri(id, friendID), {
            method: 'DELETE',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.status === 204
    }

    sendFriendRequest = async (id: string, receiverId: string, token: string): Promise<boolean> => {
        const response = await fetch(SendFriendRequestUri(id, receiverId), {
            method: 'POST',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok;
    }

    acceptFriendRequest = async (id: string, receiverId: string, token: string): Promise<boolean> => {
        const response = await fetch(AcceptFriendRequestUri(id, receiverId), {
            method: 'PUT',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok;
    }

    declineFriendRequest = async (id: string, receiverId: string, token: string): Promise<boolean> => {
        const response = await fetch(DeclineFriendRequestUri(id, receiverId), {
            method: 'DELETE',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok;
    }

    getBlockedUsers = async (id: string, token: string): Promise<UserSummary[] | null> => {
        const response = await fetch(GetBlockedUsersUri(id), {
            method: 'GET',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
            ? await response.json()
            : null;
    }

    blockUser = async (id: string, receiverId: string, token: string): Promise<boolean> => {
        const response = await fetch(BlockUserUri(id, receiverId), {
            method: 'POST',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok;
    }

    unblockUser = async (id: string, receiverId: string, token: string): Promise<boolean> => {
        const response = await fetch(UnblockUserUri(id, receiverId), {
            method: 'DELETE',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok;
    }
}

export default new SocialConnectionsService();