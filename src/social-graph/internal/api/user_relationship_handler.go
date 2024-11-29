package api

import (
	"context"
	"fmt"
	"github.com/go-chi/chi/v5"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/rs/zerolog/log"
	"net/http"
	"time"
)

func (s *Server) getCached(ctx context.Context, cacheKey string, dest interface{}) bool {
	err := s.cache.HGet(ctx, "SocialGraph", cacheKey, dest)
	return err == nil
}

func (s *Server) setCache(ctx context.Context, cacheKey string, data interface{}, expire time.Duration) {
	if err := s.cache.HSet(ctx, "SocialGraph", cacheKey, data, expire); err != nil {
		log.Error().Err(err).Msg("")
	}
}

func (s *Server) deleteCache(ctx context.Context, fields ...string) {
	if err := s.cache.HDelete(ctx, "SocialGraph", fields...); err != nil {
		log.Error().Err(err).Msg("")
	}
}

func (s *Server) deleteAllCache(ctx context.Context, key string) {
	if err := s.cache.HDeleteAll(ctx, "SocialGraph", fmt.Sprintf("*%s*", key)); err != nil {
		log.Error().Err(err).Msg("")
	}
}

func (s *Server) mapUserProfileImageUrls(ctx context.Context, user *types.User) error {
	var fileMetadata *types.FileMetadata
	cacheKey := fmt.Sprintf("images:profile:%s", user.ID)

	if ok := s.getCached(ctx, cacheKey, fileMetadata); ok {
		user.ProfileImage = &types.FileMetadataResponse{
			Url:         fileMetadata.AccessUrl.Url,
			FileName:    fileMetadata.FileName,
			ContentType: fileMetadata.ContentType,
		}
		return nil
	}

	fileMetadata, err := s.profileServiceClient.GetProfilePicture(ctx, user.ID)
	if err != nil {
		return err
	}

	s.setCache(ctx, cacheKey, fileMetadata, time.Until(fileMetadata.AccessUrl.ExpiresIn))
	user.ProfileImage = &types.FileMetadataResponse{
		Url:         fileMetadata.AccessUrl.Url,
		FileName:    fileMetadata.FileName,
		ContentType: fileMetadata.ContentType,
	}
	return nil
}

func (s *Server) mapUsersProfileImageUrls(ctx context.Context, users []*types.User) error {
	for i := range users {
		if err := s.mapUserProfileImageUrls(ctx, users[i]); err != nil {
			return err
		}
	}

	return nil
}

func (s *Server) mapAllProfileImageUrls(ctx context.Context, userWithRelationships *types.UserWithRelationships) error {
	if err := s.mapUserProfileImageUrls(ctx, userWithRelationships.User); err != nil {
		return err
	}

	userGroups := []*[]*types.User{
		&userWithRelationships.Followers,
		&userWithRelationships.Friends,
		&userWithRelationships.SentFriendRequests,
		&userWithRelationships.ReceivedFriendRequests,
		&userWithRelationships.BlockedPeople,
		&userWithRelationships.FollowedPeople,
	}

	for _, users := range userGroups {
		if err := s.mapUsersProfileImageUrls(ctx, *users); err != nil {
			return err
		}
	}

	return nil
}

func (s *Server) HandleGetUserWithRelationships(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	chi.URLParam(r, "userID")
	userID := chi.URLParam(r, "userID")

	var userWithRelationships *types.UserWithRelationships
	if ok := s.getCached(ctx, userID, userWithRelationships); ok {
		ResponseWithJSON(w, http.StatusOK, userWithRelationships)
		return nil
	}

	userWithRelationships, err := s.storage.GetUserWithRelationships(ctx, userID)
	if err != nil {
		return err
	}

	if userWithRelationships == nil {
		ResponseWithJSON(w, http.StatusNotFound, "User Not Found")
		return nil
	}

	if err = s.mapAllProfileImageUrls(ctx, userWithRelationships); err != nil {
		return err
	}

	s.setCache(ctx, userID, userWithRelationships, s.cfg.Redis.Expire)
	ResponseWithJSON(w, http.StatusOK, userWithRelationships)

	return nil
}

func (s *Server) HandleGetFollowers(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")

	var followers []*types.User
	if ok := s.getCached(ctx, GetFollowersKey(userID), &followers); ok {
		ResponseWithJSON(w, http.StatusOK, followers)
		return nil
	}

	followers, err := s.storage.GetFollowers(ctx, userID)
	if err != nil {
		return err
	}

	if err = s.mapUsersProfileImageUrls(ctx, followers); err != nil {
		return err
	}

	s.setCache(ctx, GetFollowersKey(userID), followers, s.cfg.Redis.Expire)
	ResponseWithJSON(w, http.StatusOK, followers)

	return nil
}

func (s *Server) HandleGetFollowedPeople(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")

	var followedPeople []*types.User
	if ok := s.getCached(ctx, GetFollowedKey(userID), &followedPeople); ok {
		ResponseWithJSON(w, http.StatusOK, followedPeople)
		return nil
	}

	followedPeople, err := s.storage.GetFollowedPeople(ctx, userID)
	if err != nil {
		return err
	}

	if err = s.mapUsersProfileImageUrls(ctx, followedPeople); err != nil {
		return err
	}

	s.setCache(ctx, GetFollowedKey(userID), followedPeople, s.cfg.Redis.Expire)
	ResponseWithJSON(w, http.StatusOK, followedPeople)

	return nil
}

func (s *Server) HandleGetBlockedPeople(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")

	var blockedPeople []*types.User
	if ok := s.getCached(ctx, GetBlockedKey(userID), &blockedPeople); ok {
		ResponseWithJSON(w, http.StatusOK, blockedPeople)
		return nil
	}

	blockedPeople, err := s.storage.GetBlockedPeople(ctx, userID)
	if err != nil {
		return err
	}

	if err = s.mapUsersProfileImageUrls(ctx, blockedPeople); err != nil {
		return err
	}

	s.setCache(ctx, GetBlockedKey(userID), blockedPeople, s.cfg.Redis.Expire)
	ResponseWithJSON(w, http.StatusOK, blockedPeople)

	return nil
}

func (s *Server) HandleGetFriends(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")

	var friends []*types.User
	if ok := s.getCached(ctx, GetFriendsKey(userID), &friends); ok {
		ResponseWithJSON(w, http.StatusOK, friends)
		return nil
	}

	friends, err := s.storage.GetFriends(ctx, userID)
	if err != nil {
		return err
	}

	if err = s.mapUsersProfileImageUrls(ctx, friends); err != nil {
		return err
	}

	s.setCache(ctx, GetFriendsKey(userID), friends, s.cfg.Redis.Expire)
	ResponseWithJSON(w, http.StatusOK, friends)

	return nil
}

func (s *Server) HandleGetPendingFriendRequests(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")

	var pendingFriendRequests []*types.User
	if ok := s.getCached(ctx, GetPendingRequestKey(userID), &pendingFriendRequests); ok {
		ResponseWithJSON(w, http.StatusOK, pendingFriendRequests)
		return nil
	}

	pendingFriendRequests, err := s.storage.GetPendingFriendRequests(ctx, userID)
	if err != nil {
		return err
	}

	if err = s.mapUsersProfileImageUrls(ctx, pendingFriendRequests); err != nil {
		return err
	}

	s.setCache(ctx, GetPendingRequestKey(userID), pendingFriendRequests, s.cfg.Redis.Expire)
	ResponseWithJSON(w, http.StatusOK, pendingFriendRequests)

	return nil
}

func (s *Server) HandleRemoveFriend(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	friendID := chi.URLParam(r, "friendID")

	ok, err := s.storage.RemoveFriend(ctx, userID, friendID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, "Could not remove friend relationship")
		return nil
	}

	s.deleteCache(ctx, userID, GetFriendsKey(userID), friendID, GetFriendsKey(friendID))
	if err = s.consumer.PublishMessage("user-exchange", "user.friends.remove", types.UserRemovedFriend{
		ID:       userID,
		FriendID: friendID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleFollowUser(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	followedID := chi.URLParam(r, "followedID")

	ok, err := s.storage.FollowUser(ctx, userID, followedID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, nil)
		return nil
	}

	s.deleteCache(ctx, userID, GetFollowedKey(userID), followedID, GetFollowersKey(followedID))
	if err = s.consumer.PublishMessage("user-exchange", "user.followed", types.UserFollowedMessage{
		ID:         userID,
		FollowedId: followedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleUnfollowUser(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	followedID := chi.URLParam(r, "followedID")

	ok, err := s.storage.UnfollowUser(ctx, userID, followedID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, nil)
		return nil
	}

	s.deleteCache(ctx, userID, GetFollowedKey(userID), followedID, GetFollowersKey(followedID))
	if err = s.consumer.PublishMessage("user-exchange", "user.unfollowed", types.UserUnfollowedMessage{
		ID:         userID,
		FollowedId: followedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleSendFriendRequest(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	receiverID := chi.URLParam(r, "receiverID")

	ok, err := s.storage.SendFriendRequest(ctx, userID, receiverID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, nil)
		return nil
	}

	s.deleteCache(ctx, userID, receiverID, GetPendingRequestKey(userID))
	if err = s.consumer.PublishMessage("user-exchange", "user.sent.friend", types.UserSentFriendRequestMessage{
		ID:         userID,
		ReceiverID: receiverID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleAcceptFriendRequest(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	receiverID := chi.URLParam(r, "receiverID")

	ok, err := s.storage.AcceptFriendRequest(ctx, userID, receiverID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, nil)
		return nil
	}

	s.deleteCache(ctx, userID, GetFriendsKey(userID), receiverID, GetPendingRequestKey(receiverID), GetFriendsKey(receiverID))
	if err = s.consumer.PublishMessage("user-exchange", "user.add.friend", types.UserAcceptedFriendRequestMessage{
		ID:         userID,
		ReceiverID: receiverID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleDeclineFriendRequest(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	receiverID := chi.URLParam(r, "receiverID")

	ok, err := s.storage.DeclineFriendRequest(ctx, userID, receiverID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, nil)
		return nil
	}

	s.deleteCache(ctx, userID, receiverID, GetPendingRequestKey(receiverID))
	if err = s.consumer.PublishMessage("user-exchange", "user.remove.friend", types.UserDeclinedFriendRequestMessage{
		ID:         userID,
		ReceiverID: receiverID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleBlockUser(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	blockedID := chi.URLParam(r, "blockedID")

	ok, err := s.storage.BlockUser(ctx, userID, blockedID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, nil)
		return nil
	}

	s.deleteAllCache(ctx, userID)
	s.deleteAllCache(ctx, blockedID)
	if err = s.consumer.PublishMessage("user-exchange", "user.blocked", types.UserBlockedMessage{
		ID:        userID,
		BlockedID: blockedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleUnblockUser(w http.ResponseWriter, r *http.Request) error {
	ctx := r.Context()
	userID := chi.URLParam(r, "userID")
	blockedID := chi.URLParam(r, "blockedID")

	ok, err := s.storage.UnblockUser(ctx, userID, blockedID)
	if err != nil {
		return err
	}

	if !ok {
		ResponseWithJSON(w, http.StatusNotFound, nil)
		return nil
	}

	s.deleteAllCache(ctx, userID)
	s.deleteAllCache(ctx, blockedID)
	if err = s.consumer.PublishMessage("user-exchange", "user.unblocked", types.UserUnblockedMessage{
		ID:        userID,
		BlockedID: blockedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}
