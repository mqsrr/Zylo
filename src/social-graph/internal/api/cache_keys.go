package api

import "fmt"

func GetFollowersKey(userID string) string {
	return fmt.Sprintf("%s-followers", userID)
}

func GetFollowedKey(userID string) string {
	return fmt.Sprintf("%s-followed", userID)
}

func GetBlockedKey(userID string) string {
	return fmt.Sprintf("%s-blocked", userID)
}

func GetFriendsKey(userID string) string {
	return fmt.Sprintf("%s-friends", userID)
}

func GetPendingRequestKey(userID string) string {
	return fmt.Sprintf("%s-pending-requests", userID)
}
