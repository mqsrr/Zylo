import {useEffect, useState} from "react";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {Link, useParams} from "react-router-dom";
import {User, UserSummary} from "@/models/User.ts";
import UserService from "@/services/UserService.ts";
import {Tabs} from "@radix-ui/react-tabs";
import {TabsContent, TabsList, TabsTrigger} from "@/components/ui/tabs.tsx";
import {Card, CardContent} from "@/components/ui/card.tsx";
import {Button} from "@/components/ui/button.tsx";
import SocialConnectionsService from "@/services/SocialConnectionsService.ts";

const SocialConnections = () => {
    const { id } = useParams<{ id: string; }>();
    const [user, setUser] = useState<User | null>(null);
    const { userId, accessToken } = useAuthContext();

    useEffect(() => {
        const fetchUser = async () => {
            if (id && accessToken) {
                try {
                    const fetchedUser = await UserService.getUser(id, accessToken.value, null);
                    setUser(fetchedUser);
                } catch (e) {
                    console.error("Failed to fetch user", e);
                }
            }
        };
        fetchUser();
    }, [id, accessToken]);

    if (!user) {
        return <div>Loading...</div>;
    }

    const isCurrentUser = userId === user.id;
    const { relationships } = user;

    const tabs = [
        { label: "Followers", value: "followers", users: relationships.followers || [] },
        { label: "Following", value: "following", users: relationships.followedPeople || [] },
        { label: "Friends", value: "friends", users: relationships.friends || [] },
    ];

    if (isCurrentUser) {
        tabs.push({
            label: "Sent Requests",
            value: "sentRequests",
            users: relationships.sentFriendRequests || [],
        });
        tabs.push({
            label: "Received Requests",
            value: "receivedRequests",
            users: relationships.receivedFriendRequests || [],
        });
        tabs.push({
            label: "Blocked",
            value: "blocked",
            users: relationships.blockedPeople || [],
        });
    }

    return (
        <div className="container mx-auto px-4 py-6 overflow-auto dark">
            <h2 className="text-2xl font-semibold mb-4">Connections</h2>
            <Tabs defaultValue={tabs[0].value}>
                <TabsList className="mb-4">
                    {tabs.map((tab) => (
                        <TabsTrigger key={tab.value} value={tab.value}>
                            {tab.label}
                        </TabsTrigger>
                    ))}
                </TabsList>
                {tabs.map((tab) => (
                    <TabsContent key={tab.value} value={tab.value}>
                        {tab.users.length > 0 ? (
                            tab.users.map((userSummary) => (
                                <UserCard
                                    key={userSummary.id}
                                    userSummary={userSummary}
                                    relationshipType={tab.value}
                                    isCurrentUser={isCurrentUser}
                                />
                            ))
                        ) : (
                            <p>No {tab.label.toLowerCase()} found.</p>
                        )}
                    </TabsContent>
                ))}
            </Tabs>
        </div>
    );
};

export default SocialConnections;

interface UserCardProps {
    userSummary: UserSummary;
    relationshipType: string;
    isCurrentUser: boolean;
}

const UserCard = ({ userSummary, relationshipType, isCurrentUser }: UserCardProps) => {
    const { userId, accessToken } = useAuthContext();
    const [actionState, setActionState] = useState<string>("");

    const handleAction = async () => {
        if (!userId || !accessToken) return;

        try {
            switch (relationshipType) {
                case "followers":
                    await SocialConnectionsService.followUser(userId, userSummary.id, accessToken.value);
                    setActionState("Following");
                    break;
                case "following":
                    await SocialConnectionsService.unfollowUser(userId, userSummary.id, accessToken.value);
                    setActionState("Unfollowed");
                    break;
                case "receivedRequests":
                    await SocialConnectionsService.acceptFriendRequest(userId, userSummary.id, accessToken.value);
                    setActionState("Friend Request Accepted");
                    break;
                case "blocked":
                    await SocialConnectionsService.unblockUser(userId, userSummary.id, accessToken.value);
                    setActionState("Unblocked");
                    break;
                default:
                    break;
            }
        } catch (error) {
            console.error("Action failed", error);
        }
    };

    const renderActionButton = () => {
        if (actionState) {
            return <p>{actionState}</p>;
        }

        switch (relationshipType) {
            case "followers":
                return (
                    <Button
                        onClick={handleAction}
                        variant="secondary">
                        Follow Back
                    </Button>
                );
            case "following":
                return (
                    <Button
                        onClick={handleAction}
                        variant="secondary">
                        Unfollow
                    </Button>
                );
            case "friends":
                return (
                    <Button
                        onClick={handleAction}
                        variant="secondary">
                        Remove Friend
                    </Button>
                );
            case "sentRequests":
                return (
                    <Button
                        onClick={handleAction}
                        variant="secondary">
                        Cancel Request
                    </Button>
                );
            case "receivedRequests":
                return (
                    <div className="space-x-4">
                        <Button
                            onClick={handleAction}
                            variant="secondary">
                            Accept
                        </Button>
                    </div>
                );
            case "blocked":
                return (
                    <Button
                        onClick={handleAction}
                        variant="secondary">
                        Unblock
                    </Button>
                );
            default:
                return null;
        }
    };

    return (
        <Card>
            <CardContent className="flex items-center justify-between p-4 rounded mb-4">
                <div className="flex items-center">
                    <Link to={`/profile/${userSummary.id}`}>
                        <img
                            src={userSummary.profileImage.url || "/images/default-user-image.webp"}
                            alt={userSummary.profileImage.fileName}
                            className="rounded-full w-12 h-12 object-cover"
                        />
                    </Link>
                    <div className="ml-3">
                        <p className="font-semibold text-lg">{userSummary.username}</p>
                        {<p className="text-sm text-gray-400">{"bio"}</p>}
                    </div>
                </div>
                {isCurrentUser && renderActionButton()}
            </CardContent>

        </Card>
    );
};