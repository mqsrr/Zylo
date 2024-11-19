package types

import (
	"time"
)

type UserCreatedMessage struct {
	ID string `json:"id"`
}

type UserDeletedMessage struct {
	ID string `json:"id"`
}

type UserFollowedMessage struct {
	ID         string `json:"id"`
	FollowedID string `json:"followedId"`
}

type UserUnfollowedMessage struct {
	ID         string `json:"id"`
	FollowedID string `json:"followedId"`
}

type UserAddedFriendMessage struct {
	ID       string `json:"id"`
	FriendID string `json:"friendId"`
}

type UserRemovedFriendMessage struct {
	ID       string `json:"id"`
	FriendID string `json:"friendId"`
}

type PostCreatedMessage struct {
	ID        string    `json:"id"`
	UserID    string    `json:"userId"`
	Content   string    `json:"content"`
	CreatedAt time.Time `json:"createdAt"`
}

type PostUpdatedMessage struct {
	ID      string `json:"id"`
	Content string `json:"content"`
}

type PostDeletedMessage struct {
	ID string `json:"id"`
}

type PostLikedMessage struct {
	ID     string `json:"id"`
	UserID string `json:"userId"`
}

type PostViewedMessage struct {
	ID     string `json:"id"`
	UserID string `json:"userId"`
}

type PostUnlikedMessage struct {
	ID     string `json:"id"`
	UserID string `json:"userId"`
}
