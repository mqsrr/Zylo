import PostCard from "@/components/shared/PostCard.tsx";
import {Link, useParams} from "react-router-dom";
import {useCallback, useEffect, useRef, useState} from "react";
import {User} from "@/models/User.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import UserService from "@/services/UserService.ts";
import {format} from "date-fns";
import SocialConnectionsService from "@/services/SocialConnectionsService.ts";
import {Button} from "@/components/ui/button.tsx";
import {usePostContext} from "@/hooks/usePostContext.ts";
import PostService from "@/services/PostService.ts";

interface RelationshipStatus {
    isFollowing: boolean;
    isFriend: boolean;
    hasSentFriendRequest: boolean;
    hasReceivedFriendRequest: boolean;
    isBlocked: boolean;
    hasBlocked: boolean;
}

const Profile = () => {
    const {id} = useParams<{ id: string }>();
    const {accessToken, userId} = useAuthContext();
    const {addOrUpdatePost} = usePostContext();
    const [user, setUser] = useState<User | null>(null);
    const [relationshipStatus, setRelationshipStatus] = useState<RelationshipStatus>({
        isFollowing: false,
        isFriend: false,
        hasSentFriendRequest: false,
        hasReceivedFriendRequest: false,
        isBlocked: false,
        hasBlocked: false,
    });
    const [postIds, setPostIds] = useState<string[]>([]);
    const [next, setNext] = useState<string | null>(null);
    const [isLoadingUser, setIsLoadingUser] = useState(true);
    const [isLoadingPosts, setIsLoadingPosts] = useState(false);
    const bottomRef = useRef<HTMLDivElement | null>(null);

    const isCurrentUser = id === userId || false;

    const fetchUser = useCallback(async () => {
        if (!id || !accessToken || !userId) return;

        try {
            setIsLoadingUser(true);

            const fetchedUser = await UserService.getUser(id, accessToken.value, userId);
            if (!fetchedUser) return;

            setUser(fetchedUser);
            setPostIds((prevIds) => [...prevIds, ...fetchedUser.posts.data.map((post) => post.id)]);
            setNext(fetchedUser.posts.hasNextPage ? fetchedUser.posts.next : null);
            fetchedUser.posts.data.forEach(addOrUpdatePost);

            setRelationshipStatus({
                isFollowing: fetchedUser.relationships.followers?.some((u) => u.id === userId) || false,
                isFriend: fetchedUser.relationships.friends?.some((u) => u.id === userId) || false,
                hasSentFriendRequest:
                    fetchedUser.relationships.sentFriendRequests?.some((u) => u.id === userId) || false,
                hasReceivedFriendRequest:
                    fetchedUser.relationships.receivedFriendRequests?.some((u) => u.id === userId) || false,
                isBlocked: fetchedUser.relationships.blockedPeople?.some((u) => u.id === userId) || false,
                hasBlocked: fetchedUser.relationships.blockedPeople?.some((u) => u.id === userId) || false,
            });
        } catch (error) {
            console.error("Error fetching user:", error);
        }
        finally {
            setIsLoadingUser(false);
        }
    }, [id, accessToken, userId, addOrUpdatePost]);

    const fetchPosts = useCallback(async () => {
        if (!id || !accessToken || isLoadingPosts || !next) return;

        setIsLoadingPosts(true);

        try {
            const response = await PostService.getUsersPosts(id, accessToken.value, next);
            if (response) {
                console.log("call")
                setPostIds((prevIds) => {
                    const newPostIds = response.data.map((post) => post.id);
                    return Array.from(new Set([...prevIds, ...newPostIds]));
                });
                setNext(response.hasNextPage ? response.next : null);
                response.data.forEach(addOrUpdatePost);
            }
        } catch (error) {
            console.error("Error fetching posts:", error);
        } finally {
            setIsLoadingPosts(false);
        }
    }, [id, accessToken, next, setIsLoadingPosts, addOrUpdatePost, isLoadingPosts]);

    useEffect(() => {
        fetchUser().catch(console.error);
    }, [fetchUser]);

    useEffect(() => {
        setPostIds([]);
        setNext(null);
        setUser(null);
        setRelationshipStatus({
            isFollowing: false,
            isFriend: false,
            hasSentFriendRequest: false,
            hasReceivedFriendRequest: false,
            isBlocked: false,
            hasBlocked: false,
        });
    }, [id]);

    useEffect(() => {
        const observer = new IntersectionObserver((entries) => {
            const [entry] = entries;
            if (entry.isIntersecting && next) {
                fetchPosts().catch(console.error);
            }
        }, {
            threshold: 1.0,
        });

        if (bottomRef.current) {
            observer.observe(bottomRef.current);
        }

        return () => {
            if (bottomRef.current) {
                observer.unobserve(bottomRef.current);
            }
        };
    }, [fetchPosts, next]);

    if (isLoadingUser || !user) {
        return <div>User profile is loading</div>;
    }

    const updateRelationshipState = ({
                                         isFollowing,
                                         isFriend,
                                         hasSentFriendRequest,
                                         hasReceivedFriendRequest,
                                         isBlocked,
                                         hasBlocked,
                                     }: Partial<RelationshipStatus>) => {
        setRelationshipStatus((prevStatus) => ({
            ...prevStatus,
            ...(isFollowing !== undefined && {isFollowing}),
            ...(isFriend !== undefined && {isFriend}),
            ...(hasSentFriendRequest !== undefined && {hasSentFriendRequest}),
            ...(hasReceivedFriendRequest !== undefined && {hasReceivedFriendRequest}),
            ...(isBlocked !== undefined && {isBlocked}),
            ...(hasBlocked !== undefined && {hasBlocked}),
        }));
    };


    const actionMap: { [key: string]: () => Promise<void> } = {
        follow: async () => {
            await SocialConnectionsService.followUser(userId!, user!.id, accessToken!.value);
            updateRelationshipState({isFollowing: true});
        },
        unfollow: async () => {
            await SocialConnectionsService.unfollowUser(userId!, user!.id, accessToken!.value);
            updateRelationshipState({isFollowing: false});
        },
        sendFriendRequest: async () => {
            await SocialConnectionsService.sendFriendRequest(userId!, user!.id, accessToken!.value);
            updateRelationshipState({hasSentFriendRequest: true});
        },
        acceptFriendRequest: async () => {
            await SocialConnectionsService.acceptFriendRequest(userId!, user!.id, accessToken!.value);
            updateRelationshipState({isFriend: true, hasReceivedFriendRequest: false});
        },
        unfriend: async () => {
            await SocialConnectionsService.removeFriend(userId!, user!.id, accessToken!.value);
            updateRelationshipState({isFriend: false});
        },
        block: async () => {
            await SocialConnectionsService.blockUser(userId!, user!.id, accessToken!.value);
            updateRelationshipState({hasBlocked: true});
        },
        unblock: async () => {
            await SocialConnectionsService.unblockUser(userId!, user!.id, accessToken!.value);
            updateRelationshipState({hasBlocked: false});
        },
    };


    const renderActionButtons = () => {
        if (isCurrentUser) return null;

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
                <Button
                    key="sendFriendRequest"
                    onClick={() => handleAction("sendFriendRequest")}
                    disabled
                    variant="secondary"
                >
                    Friend Request Sent
                </Button>
            );
        } else if (relationshipStatus.hasReceivedFriendRequest) {
            buttons.push(
                <Button
                    key="acceptFriendRequest"
                    onClick={() => handleAction("acceptFriendRequest")}
                >
                    Accept Friend Request
                </Button>
            );
        } else {
            buttons.push(
                <Button
                    key="sendFriendRequest"
                    onClick={() => handleAction("sendFriendRequest")}
                >
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
        const actionFunction = actionMap[actionType];
        if (!actionFunction) return;

        setIsLoadingPosts(true);
        try {
            await actionFunction();
        } catch (error) {
            console.error(`Action "${actionType}" failed`, error);
        } finally {
            setIsLoadingPosts(false);
        }
    };

    const {
        profileImage,
        backgroundImage,
        name,
        username,
        bio,
        location,
        birthDate,
        relationships,
    } = user;


    const formattedBirthDate = birthDate ? format(new Date(birthDate), "MMMM d, yyyy") : null;
    return (
        <div className="container mx-auto px-4 py-6 overflow-auto">
            <div className="relative mb-6">
                {backgroundImage && (
                    <img src={backgroundImage.url} alt={backgroundImage.fileName} className="w-full h-64 object-cover rounded-lg" />
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
                    <div className="space-x-2">{!id || id !== userId ? renderActionButtons() : null}</div>
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
                </div>
            </div>
            <div className="px-6">
                <h2 className="text-2xl font-semibold mb-4 w-full">Posts</h2>
                {postIds.length > 0 ? (
                    postIds.map((id) => <PostCard key={id} postId={id} />)
                ) : (
                    <p>This user hasn't posted anything yet.</p>
                )}
                {isLoadingPosts && <p>Loading more posts...</p>}
            </div>
        </div>
    );
};

export default Profile;
