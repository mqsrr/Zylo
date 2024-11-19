package api

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/types"
	amqp "github.com/rabbitmq/amqp091-go"
)

func (s *Server) HandlePostMessages() error {
	queues := map[string]func(amqp.Delivery) error{
		"post-created-feed-service-queue": s.handlePostCreatedMessage,
		"post-updated-feed-service-queue": s.handlePostUpdatedMessage,
		"post-deleted-feed-service-queue": s.handlePostDeletedMessage,
	}

	for queue, handler := range queues {
		if err := s.consumer.Consume(queue, s.wrapMessageHandler(handler)); err != nil {
			return err
		}
	}

	return nil
}

func (s *Server) handlePostCreatedMessage(delivery amqp.Delivery) error {
	var message types.PostCreatedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.CreatePost(context.Background(), message.ID, message.UserID, message.Content, message.CreatedAt)
}

func (s *Server) handlePostUpdatedMessage(delivery amqp.Delivery) error {
	var message types.PostUpdatedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.UpdatePostTags(context.Background(), message.ID, message.Content)
}

func (s *Server) handlePostDeletedMessage(delivery amqp.Delivery) error {
	var message types.PostDeletedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.DeletePost(context.Background(), message.ID)
}
