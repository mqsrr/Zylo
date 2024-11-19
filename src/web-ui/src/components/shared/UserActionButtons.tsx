import {Button} from "@/components/ui/button.tsx";

type UserActionButtonsProps = {
    isFollowing: boolean;
    isFriend: boolean;
    isFriendRequestSent: boolean;
    isFriendRequestReceived: boolean;
    isBlocked: boolean;
    handleFollow: () => void;
    handleSendFriendRequest: () => void;
    handleBlockUser: () => void;
};

const UserActionButtons = ({
                               isFollowing,
                               isFriend,
                               isFriendRequestSent,
                               isFriendRequestReceived,
                               isBlocked,
                               handleFollow,
                               handleSendFriendRequest,
                               handleBlockUser,
                           }: UserActionButtonsProps) => {
    return (
        <div className="flex gap-2">
            <Button onClick={handleFollow} disabled={isFollowing} variant="outline">
                {isFollowing ? "Following" : "Follow"}
            </Button>
            <Button
                onClick={handleSendFriendRequest}
                disabled={isFriend || isFriendRequestSent || isFriendRequestReceived}
                variant="outline"
            >
                {isFriend ? "Friends" : isFriendRequestSent ? "Request Sent" : isFriendRequestReceived ? "Accept Request" : "Add Friend"}
            </Button>
            <Button onClick={handleBlockUser} disabled={isBlocked} variant="outline">
                {isBlocked ? "Blocked" : "Block"}
            </Button>
        </div>
    );
};

export default UserActionButtons;