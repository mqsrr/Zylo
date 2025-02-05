package types

import (
	"github.com/oklog/ulid/v2"
)

type Relationship struct {
	IDs       []ulid.ULID          `json:"ids"`
	CreatedAt map[ulid.ULID]string `json:"createdAt"`
}

type FollowRequests struct {
	Followers *Relationship `json:"followers"`
	Following *Relationship `json:"following"`
}

type FriendRequests struct {
	Sent     *Relationship `json:"sent"`
	Received *Relationship `json:"received"`
}

type UserWithRelationships struct {
	UserID         ulid.ULID       `json:"userId"`
	Friends        *Relationship   `json:"friends"`
	Follows        *FollowRequests `json:"follows"`
	Blocks         *Relationship   `json:"blocks"`
	FriendRequests *FriendRequests `json:"friendRequests"`
}

type UserCreatedMessage struct {
	ID ulid.ULID `json:"id"`
}

type UserUpdatedMessage struct {
	ID ulid.ULID `json:"id"`
}

type UserDeletedMessage struct {
	ID ulid.ULID `json:"id"`
}

type UserFollowedMessage struct {
	ID         ulid.ULID `json:"id"`
	FollowedId ulid.ULID `json:"followedId"`
}

type UserUnfollowedMessage struct {
	ID         ulid.ULID `json:"id"`
	FollowedId ulid.ULID `json:"followedId"`
}
type UserSentFriendRequestMessage struct {
	ID         ulid.ULID `json:"id"`
	ReceiverID ulid.ULID `json:"receiverId"`
}
type UserAcceptedFriendRequestMessage struct {
	ID         ulid.ULID `json:"id"`
	ReceiverID ulid.ULID `json:"receiverId"`
}
type UserDeclinedFriendRequestMessage struct {
	ID         ulid.ULID `json:"id"`
	ReceiverID ulid.ULID `json:"receiverId"`
}

type UserRemovedFriend struct {
	ID       ulid.ULID `json:"id"`
	FriendID ulid.ULID `json:"friendId"`
}

type UserBlockedMessage struct {
	ID        ulid.ULID `json:"id"`
	BlockedID ulid.ULID `json:"blockedID"`
}
type UserUnblockedMessage struct {
	ID        ulid.ULID `json:"id"`
	BlockedID ulid.ULID `json:"blockedID"`
}
