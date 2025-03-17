package api

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/types"
	amqp "github.com/rabbitmq/amqp091-go"
)

func (s *Server) HandleUserMessages() error {
	queues := map[string]func(amqp.Delivery) error{
		"user-created-feed-service-queue": s.handleUserCreatedMessage,
		"user-deleted-feed-service-queue": s.handleUserDeletedMessage,
	}

	for queue, handler := range queues {
		if err := s.consumer.Consume(queue, s.wrapMessageHandler(handler)); err != nil {
			return err
		}
	}

	return nil
}

func (s *Server) handleUserCreatedMessage(delivery amqp.Delivery) error {
	var message types.UserCreatedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.userRepository.CreateUser(context.Background(), message.ID, message.CreatedAt)
}

func (s *Server) handleUserDeletedMessage(delivery amqp.Delivery) error {
	var message types.UserDeletedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.userRepository.DeleteUser(context.Background(), message.ID)
}
