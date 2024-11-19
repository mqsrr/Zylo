package api

import (
	"context"
	"encoding/json"
	"github.com/mqsrr/zylo/feed-service/internal/types"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/rs/zerolog/log"
)

func (s *Server) HandleUserSocialMessages() error {
	queues := map[string]func(amqp.Delivery) error{
		"user-followed-feed-service-queue":      s.handleUserFollowedMessage,
		"user-unfollowed-feed-service-queue":    s.handleUserUnfollowedMessage,
		"user-add-friend-feed-service-queue":    s.handleAddFriendMessage,
		"user-remove-friend-feed-service-queue": s.handleUserRemovedFriendMessage,
	}

	for queue, handler := range queues {
		if err := s.consumer.Consume(queue, s.wrapMessageHandler(handler)); err != nil {
			return err
		}
	}

	return nil
}

func (s *Server) wrapMessageHandler(handler func(amqp.Delivery) error) func(amqp.Delivery) {
	return func(delivery amqp.Delivery) {
		if err := handler(delivery); err != nil {
			log.Error().Err(err).Msg("Message processing failed")
		}

		if err := delivery.Ack(false); err != nil {
			log.Error().Err(err).Msg("Failed to acknowledge message")
		}
	}
}

func unmarshalMessage[T any](delivery amqp.Delivery, target T) error {
	if err := json.Unmarshal(delivery.Body, target); err != nil {
		return err
	}
	return nil
}

func (s *Server) handleUserFollowedMessage(delivery amqp.Delivery) error {
	var message types.UserFollowedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.FollowUser(context.Background(), message.ID, message.FollowedID)
}

func (s *Server) handleUserUnfollowedMessage(delivery amqp.Delivery) error {
	var message types.UserUnfollowedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.UnfollowUser(context.Background(), message.ID, message.FollowedID)
}

func (s *Server) handleAddFriendMessage(delivery amqp.Delivery) error {
	var message types.UserAddedFriendMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.AddFriend(context.Background(), message.ID, message.FriendID)
}

func (s *Server) handleUserRemovedFriendMessage(delivery amqp.Delivery) error {
	var message types.UserRemovedFriendMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.RemoveFriend(context.Background(), message.ID, message.FriendID)
}
