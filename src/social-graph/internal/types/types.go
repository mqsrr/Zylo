package types

import (
	"github.com/oklog/ulid/v2"
	"time"
)

type User struct {
	ID           ulid.ULID     `json:"id"`
	Username     string        `json:"username"`
	ProfileImage *FileMetadata `json:"profileImage"`
	Name         string        `json:"name"`
	Bio          string        `json:"bio"`
	Location     string        `json:"location"`
	CreatedAt    string        `json:"createdAt"`
}
type UserWithRelationships struct {
	User                   *User
	Followers              []*User
	FollowedPeople         []*User
	BlockedPeople          []*User
	Friends                []*User
	SentFriendRequests     []*User
	ReceivedFriendRequests []*User
}

type FileMetadata struct {
	AccessUrl   *PresignedUrl `json:"accessUrl"`
	FileName    string        `json:"fileName"`
	ContentType string        `json:"contentType"`
}

type PresignedUrl struct {
	Url       string `json:"url"`
	ExpiresIn time.Time
}

type UserCreatedMessage struct {
	ID       string `json:"id"`
	Username string `json:"username"`
	Name     string `json:"name"`
}

type UserUpdatedMessage struct {
	ID       string `json:"id"`
	Name     string `json:"name"`
	Bio      string `json:"bio"`
	Location string `json:"location"`
}

type UserDeletedMessage struct {
	ID string `json:"id"`
}

type UserFollowedMessage struct {
	ID         string `json:"id"`
	FollowedId string `json:"followedId"`
}

type UserUnfollowedMessage struct {
	ID         string `json:"id"`
	FollowedId string `json:"followedId"`
}
type UserSentFriendRequestMessage struct {
	ID         string `json:"id"`
	ReceiverID string `json:"receiverId"`
}
type UserAcceptedFriendRequestMessage struct {
	ID         string `json:"id"`
	ReceiverID string `json:"receiverId"`
}
type UserDeclinedFriendRequestMessage struct {
	ID         string `json:"id"`
	ReceiverID string `json:"receiverId"`
}

type UserRemovedFriend struct {
	ID       string `json:"id"`
	FriendID string `json:"friendId"`
}

type UserBlockedMessage struct {
	ID        string `json:"id"`
	BlockedID string `json:"blockedID"`
}
type UserUnblockedMessage struct {
	ID        string `json:"id"`
	BlockedID string `json:"blockedID"`
}
