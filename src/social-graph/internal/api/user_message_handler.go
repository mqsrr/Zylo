package api

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/rs/zerolog/log"
)

func (s *Server) HandleUserCreatedMessage() error {
	return s.consumer.Consume("user-created-social-graph-queue", s.handleUserCreatedMessage)
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

	if _, err := s.storage.CreateUser(context.Background(), userMsg.ID); err != nil {
		log.Error().
			Timestamp().
			Caller().
			Str("user_id", userMsg.ID.String()).
			Str("queue", "user-created-social-graph-queue").
			Str("exchange", "user-exchange").
			Err(err).
			Msg("")

		if err = delivery.Nack(false, false); err != nil {
			log.Error().Err(err).Msg("")
		}

		return
	}

	if err := delivery.Ack(false); err != nil {
		log.Error().
			Timestamp().
			Caller().
			Str("user_id", userMsg.ID.String()).
			Str("queue", "user-created-social-graph-queue").
			Str("exchange", "user-exchange").
			Err(err).Msg("")
	}
}

func (s *Server) handleUserDeletedMessage(delivery amqp.Delivery) {
	var userMsg types.UserDeletedMessage
	if err := json.Unmarshal(delivery.Body, &userMsg); err != nil {
		log.Error().Err(err).Msg("Could not decode queue message")
		return
	}

	ctx := context.Background()
	if _, err := s.storage.DeleteUserByID(ctx, userMsg.ID); err != nil {
		log.Error().
			Timestamp().
			Caller().
			Str("user_id", userMsg.ID.String()).
			Str("queue", "user-deleted-social-graph-queue").
			Str("exchange", "user-exchange").
			Err(err).Msg("")

		if err = delivery.Nack(false, false); err != nil {
			log.Error().Err(err).Msg("")
		}

		return
	}

	if err := delivery.Ack(false); err != nil {
		log.Error().
			Timestamp().
			Caller().
			Str("user_id", userMsg.ID.String()).
			Str("queue", "user-created-social-graph-queue").
			Str("exchange", "user-exchange").
			Err(err).Msg("")
	}

	if err := s.cache.HDeleteAll(ctx, "SocialGraph", fmt.Sprintf("*%s*", userMsg.ID)); err != nil {
		log.Error().Err(err).Msg("")
	}
}
