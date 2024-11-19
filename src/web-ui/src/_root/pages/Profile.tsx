import PostCard from "@/components/shared/PostCard.tsx";
import {Link, useParams} from "react-router-dom";
import {useCallback, useEffect, useState} from "react";
import {User, UserSummary} from "@/models/User.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import UserService from "@/services/UserService.ts";
import {format} from "date-fns";
import SocialConnectionsService from "@/services/SocialConnectionsService.ts";
import {useUserContext} from "@/hooks/useTokenContext.ts";
import {Button} from "@/components/ui/button.tsx";

interface RelationshipStatus {
    isFollowing: boolean;
    isFriend: boolean;
    hasSentFriendRequest: boolean;
    hasReceivedFriendRequest: boolean;
    isBlocked: boolean; // The profile user has blocked the current user
    hasBlocked: boolean; // The current user has blocked the profile user
}

const Profile = () => {
    const {id} = useParams<{ id: string }>();
    const {accessToken, userId} = useAuthContext();
    const {user: currentUser} = useUserContext();
    const [user, setUser] = useState<User | null>(null);
    const [relationshipStatus, setRelationshipStatus] = useState<RelationshipStatus>({
        isFollowing: false,
        isFriend: false,
        hasSentFriendRequest: false,
        hasReceivedFriendRequest: false,
        isBlocked: false,
        hasBlocked: false,
    });
    const [isActionLoading, setIsActionLoading] = useState<boolean>(false);

    const fetchUser = useCallback(async () => {
        if (!id || !accessToken || !userId || !currentUser) {
            return;
        }

        try {
            const fetchedUser = await UserService.getUser(id, accessToken.value, userId);
            if (!fetchedUser) {
                return null;
            }
            setUser(fetchedUser);

            setRelationshipStatus({
                isFollowing: fetchedUser.relationships.followers?.some(u => u.id === userId) || false,
                isFriend: fetchedUser.relationships.friends?.some(u => u.id === userId) || false,
                hasSentFriendRequest: currentUser.relationships.sentFriendRequests?.some(u => u.id === fetchedUser.id) || false,
                hasReceivedFriendRequest: fetchedUser.relationships.receivedFriendRequests?.some(u => u.id === userId) || false,
                isBlocked: fetchedUser.relationships.blockedPeople?.some(u => u.id === userId) || false,
                hasBlocked: currentUser.relationships.blockedPeople?.some(u => u.id === fetchedUser.id) || false,
            });
        } catch (error) {
            console.error("Error fetching user:", error);
        }
    }, [id, accessToken, userId, currentUser]);

    useEffect(() => {
        if (!id || !currentUser) {
            return;
        }

        if (id === currentUser.id) {
            setUser(currentUser);
            setRelationshipStatus({
                isFollowing: false,
                isFriend: false,
                hasSentFriendRequest: false,
                hasReceivedFriendRequest: false,
                isBlocked: false,
                hasBlocked: false,
            });
            return;
        }

        fetchUser().catch(console.error);
    }, [id, currentUser, fetchUser]);

    if (!user) {
        return <div>Loading</div>;
    }

    const {
        profileImage,
        backgroundImage,
        name,
        username,
        bio,
        location,
        birthDate,
        relationships,
        posts,
    } = user;

    const formattedBirthDate = birthDate
        ? format(new Date(birthDate), "MMMM d, yyyy")
        : null;

    const isCurrentUser = id === currentUser?.id || false;
    const currentUserSummary: UserSummary | null = currentUser
        ? {
            id: currentUser.id,
            profileImage: currentUser.profileImage,
            name: currentUser.name,
            username: currentUser.username,
            bio: currentUser.bio,
            location: currentUser.location,
        } : null;

    const actionMap: { [key: string]: () => Promise<void> } = {
        follow: async () => {
            await SocialConnectionsService.followUser(userId!, user.id, accessToken!.value);
            setRelationshipStatus((prevStatus) => ({
                ...prevStatus,
                isFollowing: true,
            }));
            if (currentUserSummary) {
                setUser((prevUser) => {
                    if (!prevUser) {
                        return prevUser;
                    }

                    const newFollowers = prevUser.relationships.followers
                        ? [...prevUser.relationships.followers, currentUserSummary]
                        : [currentUserSummary];
                    return {
                        ...prevUser,
                        relationships: {
                            ...prevUser.relationships,
                            followers: newFollowers,
                        },
                    };
                });
            }
        },
        unfollow: async () => {
            await SocialConnectionsService.unfollowUser(userId!, user.id, accessToken!.value);
            setRelationshipStatus((prevStatus) => ({
                ...prevStatus,
                isFollowing: false,
            }));
            setUser((prevUser) => {
                if (!prevUser) {
                    return prevUser;
                }

                const newFollowers = prevUser.relationships.followers?.filter(
                    (follower) => follower.id !== userId
                ) || [];

                return {
                    ...prevUser,
                    relationships: {
                        ...prevUser.relationships,
                        followers: newFollowers,
                    },
                };
            });
        },
        sendFriendRequest: async () => {
            await SocialConnectionsService.sendFriendRequest(userId!, user.id, accessToken!.value);
            setRelationshipStatus((prevStatus) => ({
                ...prevStatus,
                hasSentFriendRequest: true,
            }));
            if (currentUserSummary) {
                setUser((prevUser) => {
                    if (!prevUser) {
                        return prevUser;
                    }

                    const newReceivedRequests = prevUser.relationships.receivedFriendRequests
                        ? [...prevUser.relationships.receivedFriendRequests, currentUserSummary]
                        : [currentUserSummary];
                    return {
                        ...prevUser,
                        relationships: {
                            ...prevUser.relationships,
                            receivedFriendRequests: newReceivedRequests,
                        },
                    };
                });
            }
        },
        acceptFriendRequest: async () => {
            await SocialConnectionsService.acceptFriendRequest(userId!, user.id, accessToken!.value);
            setRelationshipStatus((prevStatus) => ({
                ...prevStatus,
                isFriend: true,
                hasReceivedFriendRequest: false,
            }));
            if (currentUserSummary) {
                setUser((prevUser) => {
                    if (!prevUser) {
                        return prevUser;
                    }

                    const newFriends = prevUser.relationships.friends
                        ? [...prevUser.relationships.friends, currentUserSummary]
                        : [currentUserSummary];
                    const newReceivedRequests = prevUser.relationships.receivedFriendRequests?.filter(
                        (requester) => requester.id !== currentUserSummary.id
                    ) || [];
                    return {
                        ...prevUser,
                        relationships: {
                            ...prevUser.relationships,
                            friends: newFriends,
                            receivedFriendRequests: newReceivedRequests,
                        },
                    };
                });
            }
        },
        unfriend: async () => {
            await SocialConnectionsService.removeFriend(userId!, user.id, accessToken!.value);
            setRelationshipStatus((prevStatus) => ({
                ...prevStatus,
                isFriend: false,
            }));
            setUser((prevUser) => {
                if (!prevUser) {
                    return prevUser;
                }

                const newFriends = prevUser.relationships.friends?.filter(
                    (friend) => friend.id !== userId
                ) || [];
                return {
                    ...prevUser,
                    relationships: {
                        ...prevUser.relationships,
                        friends: newFriends,
                    },
                };
            });
        },
        block: async () => {
            await SocialConnectionsService.blockUser(userId!, user.id, accessToken!.value);
            setRelationshipStatus({
                isFollowing: false,
                isFriend: false,
                hasSentFriendRequest: false,
                hasReceivedFriendRequest: false,
                isBlocked: false,
                hasBlocked: true,
            });
            if (currentUserSummary) {
                setUser((prevUser) => {
                    if (!prevUser) {
                        return prevUser;
                    }

                    const newFollowers = prevUser.relationships.followers?.filter(u => u.id !== userId) || [];
                    const newFriends = prevUser.relationships.friends?.filter(u => u.id !== userId) || [];
                    const newReceivedFriendRequests = prevUser.relationships.receivedFriendRequests?.filter(u => u.id !== userId) || [];
                    const newBlockedPeople = prevUser.relationships.blockedPeople
                        ? [...prevUser.relationships.blockedPeople, currentUserSummary]
                        : [currentUserSummary];
                    return {
                        ...prevUser,
                        relationships: {
                            ...prevUser.relationships,
                            followers: newFollowers,
                            friends: newFriends,
                            receivedFriendRequests: newReceivedFriendRequests,
                            blockedPeople: newBlockedPeople,
                        },
                    };
                });
            }
        },
        unblock: async () => {
            await SocialConnectionsService.unblockUser(userId!, user.id, accessToken!.value);
            setRelationshipStatus((prevStatus) => ({
                ...prevStatus,
                hasBlocked: false,
            }));
            setUser((prevUser) => {
                if (!prevUser) {
                    return prevUser;
                }

                const newBlockedPeople = prevUser.relationships.blockedPeople?.filter((blocked) => blocked.id !== userId) || [];
                return {
                    ...prevUser,
                    relationships: {
                        ...prevUser.relationships,
                        blockedPeople: newBlockedPeople,
                    },
                };
            });
        },
    };

    const renderActionButtons = () => {
        if (isActionLoading) {
            return <p>Processing...</p>;
        }

        if (relationshipStatus.hasBlocked) {
            return (
                <Button onClick={() => handleAction("unblock")} variant="secondary">
                    Unblock
                </Button>
            );
        }

        if (relationshipStatus.isBlocked) {
            return <p>You have been blocked by this user.</p>;
        }

        const buttons = [];
        if (relationshipStatus.isFollowing) {
            buttons.push(
                <Button key="unfollow" onClick={() => handleAction("unfollow")} variant="secondary">
                    Unfollow
                </Button>
            );
        } else {
            buttons.push(
                <Button key="follow" onClick={() => handleAction("follow")}>
                    Follow
                </Button>
            );
        }

        if (relationshipStatus.isFriend) {
            buttons.push(
                <Button key="unfriend" onClick={() => handleAction("unfriend")} variant="destructive">
                    Unfriend
                </Button>
            );
        } else if (relationshipStatus.hasSentFriendRequest) {
            buttons.push(
                <Button key="sendFriendRequest" onClick={() => handleAction("sendFriendRequest")} disabled={true}
                        variant="secondary">
                    Friend Request Sent
                </Button>
            );
        } else if (relationshipStatus.hasReceivedFriendRequest) {
            buttons.push(
                <Button key="acceptFriendRequest" onClick={() => handleAction("acceptFriendRequest")}>
                    Accept Friend Request
                </Button>
            );
        } else {
            buttons.push(
                <Button key="sendFriendRequest" onClick={() => handleAction("sendFriendRequest")}>
                    Send Friend Request
                </Button>
            );
        }

        buttons.push(
            <Button key="block" onClick={() => handleAction("block")} variant="destructive">
                Block
            </Button>
        );

        return <>{buttons}</>;
    };

    const handleAction = async (actionType: string) => {
        if (!userId || !accessToken) return;

        const actionFunction = actionMap[actionType];
        if (!actionFunction) {
            console.error(`Action type "${actionType}" is not defined.`);
            return;
        }

        setIsActionLoading(true);

        try {
            await actionFunction();
        } catch (error) {
            console.error(`Action "${actionType}" failed`, error);
        } finally {
            setIsActionLoading(false);
        }
    };

    return (
        <div className="container mx-auto px-4 py-6 overflow-auto">

            <div className="relative mb-6">
                {backgroundImage && (
                    <img
                        src={backgroundImage.url}
                        alt={backgroundImage.fileName}
                        className="w-full h-64 object-cover rounded-lg"
                    />
                )}
                <div className="absolute -bottom-12 left-6">
                    {profileImage && (
                        <img
                            src={profileImage.url}
                            alt={profileImage.fileName}
                            className="w-24 h-24 object-cover rounded-full border-4 border-white"
                        />
                    )}
                </div>
            </div>

            <div className="mt-16 mb-8 px-6">
                <h1 className="text-3xl font-bold">{name}</h1>
                <div className="flex items-center justify-between">
                    <h2 className="text-xl font-bold text-gray-600">@{username}</h2>
                    {!isCurrentUser && (
                        <div className="flex gap-4">{renderActionButtons()}</div>
                    )}
                </div>

                {location && (
                    <p className="text-gray-600 mt-2">
                        <strong>Location:</strong> {location}
                    </p>
                )}
                {formattedBirthDate && (
                    <p className="text-gray-600 mt-1">
                        <strong>Birth Date:</strong> {formattedBirthDate}
                    </p>
                )}
                {bio && <p className="mt-4">{bio}</p>}

                <div className="flex gap-4 mt-4">
                    <Link to={`/profile/${id}/socials`} className="hover:underline flex items-center">
                        <strong className="mr-1">{relationships.followers?.length || 0}</strong>
                        <span className="text-gray-500">Followers</span>
                    </Link>
                    <Link to={`/profile/${id}/socials`} className="hover:underline flex items-center">
                        <strong className="mr-1">{relationships.followedPeople?.length || 0}</strong>
                        <span className="text-gray-500">Following</span>
                    </Link>
                    <Link to={`/profile/${id}/socials`} className="hover:underline flex items-center">
                        <strong className="mr-1">{relationships.friends?.length || 0}</strong>
                        <span className="text-gray-500">Friends</span>
                    </Link>
                </div>
            </div>

            <div className="px-6">
                <h2 className="text-2xl font-semibold mb-4 w-full">Posts</h2>
                {posts && posts.data.length > 0 ? (
                    posts.data.map((post) => <PostCard key={post.id} post={post}/>)
                ) : (
                    <p>This user hasn't posted anything yet.</p>
                )}
            </div>
        </div>
    );
};

export default Profile;
