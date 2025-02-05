package api

import (
	"github.com/go-chi/chi/v5"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"net/http"
)

func (s *Server) HandleGetUserWithRelationships(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))

	userWithRelationships, err := s.storage.GetUserWithRelationships(r.Context(), id)
	if err != nil {
		return err
	}

	if userWithRelationships == nil {
		return types.NewNotFound("User could not be found")
	}

	ResponseWithJSON(w, http.StatusOK, userWithRelationships)
	return nil
}

func (s *Server) HandleGetFollowers(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))

	followers, err := s.storage.GetFollowers(r.Context(), id)
	if err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusOK, followers)
	return nil
}

func (s *Server) HandleGetFollowedPeople(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))

	followedPeople, err := s.storage.GetFollowedPeople(r.Context(), id)
	if err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusOK, followedPeople)
	return nil
}

func (s *Server) HandleGetBlockedPeople(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))

	blockedPeople, err := s.storage.GetBlockedPeople(r.Context(), id)
	if err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusOK, blockedPeople)
	return nil
}

func (s *Server) HandleGetFriends(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))

	friends, err := s.storage.GetFriends(r.Context(), id)
	if err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusOK, friends)
	return nil
}

func (s *Server) HandleGetPendingFriendRequests(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))

	pendingFriendRequests, err := s.storage.GetPendingFriendRequests(r.Context(), id)
	if err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusOK, pendingFriendRequests)
	return nil
}

func (s *Server) HandleRemoveFriend(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	friendID := ulid.MustParse(chi.URLParam(r, "friendId"))

	ok, err := s.storage.RemoveFriend(r.Context(), id, friendID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewNotFound("Relation or user could not be found")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.friends.remove", types.UserRemovedFriend{
		ID:       id,
		FriendID: friendID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleFollowUser(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	followedID := ulid.MustParse(chi.URLParam(r, "followedId"))

	ok, err := s.storage.FollowUser(r.Context(), id, followedID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewBadRequest("Relation already exists or user does not exists")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.followed", types.UserFollowedMessage{
		ID:         id,
		FollowedId: followedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleUnfollowUser(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	followedID := ulid.MustParse(chi.URLParam(r, "followedId"))

	ok, err := s.storage.UnfollowUser(r.Context(), id, followedID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewBadRequest("Relation or user does not exists")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.unfollowed", types.UserUnfollowedMessage{
		ID:         id,
		FollowedId: followedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleSendFriendRequest(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	receiverID := ulid.MustParse(chi.URLParam(r, "receiverId"))

	ok, err := s.storage.SendFriendRequest(r.Context(), id, receiverID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewBadRequest("Relation already exists or user does not exists")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.sent.friend", types.UserSentFriendRequestMessage{
		ID:         id,
		ReceiverID: receiverID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleAcceptFriendRequest(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	receiverID := ulid.MustParse(chi.URLParam(r, "receiverId"))

	ok, err := s.storage.AcceptFriendRequest(r.Context(), id, receiverID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewBadRequest("Could not accept friend request due to unmet constraints. Make sure friend request exists and user id is correct")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.add.friend", types.UserAcceptedFriendRequestMessage{
		ID:         id,
		ReceiverID: receiverID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleDeclineFriendRequest(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	receiverID := ulid.MustParse(chi.URLParam(r, "receiverId"))

	ok, err := s.storage.DeclineFriendRequest(r.Context(), id, receiverID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewBadRequest("Relation or user does not exists")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.remove.friend", types.UserDeclinedFriendRequestMessage{
		ID:         id,
		ReceiverID: receiverID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleBlockUser(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	blockedID := ulid.MustParse(chi.URLParam(r, "blockedId"))

	ok, err := s.storage.BlockUser(r.Context(), id, blockedID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewBadRequest("Relation already exists or user does not exists")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.blocked", types.UserBlockedMessage{
		ID:        id,
		BlockedID: blockedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}

func (s *Server) HandleUnblockUser(w http.ResponseWriter, r *http.Request) error {
	id := ulid.MustParse(chi.URLParam(r, "id"))
	blockedID := ulid.MustParse(chi.URLParam(r, "blockedId"))

	ok, err := s.storage.UnblockUser(r.Context(), id, blockedID)
	if err != nil {
		return err
	}

	if !ok {
		return types.NewBadRequest("Relation or user does not exists")
	}

	if err = s.consumer.PublishMessage("user-exchange", "user.unblocked", types.UserUnblockedMessage{
		ID:        id,
		BlockedID: blockedID,
	}); err != nil {
		return err
	}

	ResponseWithJSON(w, http.StatusNoContent, nil)
	return nil
}
