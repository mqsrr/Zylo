package api

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/rs/zerolog/log"
	"time"
)

func (s *Server) HandleUserCreatedMessage() error {
	return s.consumer.Consume("user-created-social-graph-queue", s.handleUserCreatedMessage)
}

func (s *Server) HandleUserUpdatedMessage() error {
	return s.consumer.Consume("user-updated-social-graph-queue", s.handleUserUpdatedMessage)
}

func (s *Server) HandleUserDeletedMessage() error {
	return s.consumer.Consume("user-deleted-social-graph-queue", s.handleUserDeletedMessage)
}

func (s *Server) handleUserCreatedMessage(delivery amqp.Delivery) {
	var userMsg types.UserCreatedMessage
	if err := json.Unmarshal(delivery.Body, &userMsg); err != nil {
		log.Error().Err(err).Msg("Could not decode queue message")
		return
	}

	user := &types.User{
		ID:        ulid.MustParse(userMsg.ID),
		Username:  userMsg.Username,
		CreatedAt: time.Now().UTC().Format(time.RFC3339),
	}

	_, err := s.storage.CreateUser(context.Background(), user)
	if err != nil {
		log.Error().Err(err).Msg("")
		err = delivery.Nack(false, true)

		log.Error().Err(err).Msg("")
		return
	}

	err = delivery.Ack(false)
	if err != nil {
		log.Error().Err(err).Msg("")
	}
}

func (s *Server) handleUserUpdatedMessage(delivery amqp.Delivery) {
	var msg types.UserUpdatedMessage
	if err := json.Unmarshal(delivery.Body, &msg); err != nil {
		log.Error().Err(err).Msg("Could not decode queue message")
		return
	}

	ctx := context.Background()
	_, err := s.storage.UpdateUser(ctx, msg.ID, msg.Name, msg.Bio, msg.Location)
	if err != nil {
		log.Error().Err(err).Msg("")
		err = delivery.Nack(false, true)

		log.Error().Err(err).Msg("")
		return
	}

	if err = delivery.Ack(false); err != nil {
		log.Error().Err(err).Msg("")
	}

	if err = s.cache.HDeleteAll(ctx, "SocialGraph", fmt.Sprintf("*%s*", msg.ID)); err != nil {
		log.Error().Err(err).Msg("")
	}
}

func (s *Server) handleUserDeletedMessage(delivery amqp.Delivery) {
	var userMsg types.UserDeletedMessage
	if err := json.Unmarshal(delivery.Body, &userMsg); err != nil {
		log.Error().Err(err).Msg("Could not decode queue message")
		return
	}

	ctx := context.Background()
	_, err := s.storage.DeleteUserByID(ctx, userMsg.ID)
	if err != nil {
		log.Error().Err(err).Msg("")
		err = delivery.Nack(false, true)

		log.Error().Err(err).Msg("")
		return
	}

	if err = delivery.Ack(false); err != nil {
		log.Error().Err(err).Msg("")
	}

	if err = s.cache.HDeleteAll(ctx, "SocialGraph", fmt.Sprintf("*%s*", userMsg.ID)); err != nil {
		log.Error().Err(err).Msg("")
	}
}
