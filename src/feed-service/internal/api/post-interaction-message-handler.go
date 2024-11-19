package api

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/types"
	amqp "github.com/rabbitmq/amqp091-go"
)

func (s *Server) HandlePostInteractionMessages() error {
	queues := map[string]func(amqp.Delivery) error{
		"post-liked-feed-service-queue":   s.handlePostLikedMessage,
		"post-unliked-feed-service-queue": s.handlePostUnlikedMessage,
		"post-viewed-feed-service-queue":  s.handlePostViewedMessage,
	}

	for queue, handler := range queues {
		if err := s.consumer.Consume(queue, s.wrapMessageHandler(handler)); err != nil {
			return err
		}
	}

	return nil
}

func (s *Server) handlePostLikedMessage(delivery amqp.Delivery) error {
	var message types.PostLikedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.LikePost(context.Background(), message.UserID, message.ID)
}

func (s *Server) handlePostViewedMessage(delivery amqp.Delivery) error {
	var message types.PostViewedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.ViewPost(context.Background(), message.UserID, message.ID)
}

func (s *Server) handlePostUnlikedMessage(delivery amqp.Delivery) error {
	var message types.PostUnlikedMessage
	if err := unmarshalMessage(delivery, &message); err != nil {
		return err
	}

	return s.storage.UnlikePost(context.Background(), message.UserID, message.ID)
}
