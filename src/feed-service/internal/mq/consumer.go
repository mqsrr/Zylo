package mq

import (
	"encoding/json"
	"fmt"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/rs/zerolog/log"
	"os"
	"os/signal"
	"strings"
	"syscall"
)

type Consumer interface {
	Consume(queue string, consumeFunc func(delivery amqp.Delivery)) error
	PublishMessage(exchangeName, routingKey string, v interface{}) error
	Shutdown() error
}

type AmqConsumer struct {
	conn    *amqp.Connection
	channel *amqp.Channel
	tag     string
	done    chan error
}

func SetupCloseHandler(consumer Consumer) {
	c := make(chan os.Signal, 2)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)

	go func() {
		<-c
		log.Info().Msg("Ctrl+C pressed in Terminal")
		if err := consumer.Shutdown(); err != nil {
			log.Error().Err(err).Msgf("error during shutdown: %s", err)
		}
		os.Exit(0)
	}()
}

func (c *AmqConsumer) Shutdown() error {
	if err := c.channel.Cancel(c.tag, true); err != nil {
		return fmt.Errorf("consumer cancel failed: %s", err)
	}

	if err := c.conn.Close(); err != nil {
		return fmt.Errorf("AMQP connection close error: %s", err)
	}

	defer log.Info().Msg("AMQP shutdown OK")
	return nil
}

func NewConsumer(cfg *config.RabbitmqConfig) (Consumer, error) {
	var err error
	c := &AmqConsumer{
		tag:  "",
		done: make(chan error),
	}

	config := amqp.Config{Properties: amqp.NewConnectionProperties()}
	config.Properties.SetClientConnectionName(cfg.ConnName)

	c.conn, err = amqp.DialConfig(cfg.AmqpURI, config)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to RabbitMQ: %v", err)
	}

	c.channel, err = c.conn.Channel()
	if err != nil {
		return nil, fmt.Errorf("failed to open a channel: %v", err)
	}

	if err = declareExchanges(c.channel, "user-exchange", "post-exchange"); err != nil {
		return nil, err
	}

	if err = declareQueues(c.channel); err != nil {
		return nil, err
	}

	SetupCloseHandler(c)
	return c, nil
}

func (c *AmqConsumer) Consume(queue string, consumeFunc func(delivery amqp.Delivery)) error {
	deliveries, err := c.channel.Consume(
		queue,
		c.tag,
		false,
		false,
		false,
		false,
		nil,
	)
	if err != nil {
		return fmt.Errorf("error creating new consumer: %s", err)
	}

	go c.handleMessages(deliveries, consumeFunc)
	return nil
}

func (c *AmqConsumer) PublishMessage(exchangeName, routingKey string, v interface{}) error {
	body, err := json.Marshal(v)
	if err != nil {
		return err
	}

	err = c.channel.Publish(
		exchangeName,
		routingKey,
		false,
		false,
		amqp.Publishing{
			ContentType: "application/json",
			Body:        body,
		},
	)

	if err != nil {
		return fmt.Errorf("error publishing message: %s", err)
	}

	log.Debug().Msgf("Message published to routing key: %s", routingKey)
	return nil
}

func declareExchanges(channel *amqp.Channel, exchangeNames ...string) error {
	for i := range exchangeNames {
		if err := channel.ExchangeDeclare(
			exchangeNames[i],
			"direct",
			true,
			false,
			false,
			false,
			nil); err != nil {
			return err
		}
	}

	return nil
}

func declareAndBindQueue(channel *amqp.Channel, queueName, routingKey, exchangeName string) error {
	queue, err := channel.QueueDeclare(
		queueName,
		true,
		false,
		false,
		false,
		nil,
	)
	if err != nil {
		return fmt.Errorf("queue Declare: %s", err)
	}

	if err = channel.QueueBind(
		queue.Name,
		routingKey,
		exchangeName,
		false,
		nil,
	); err != nil {
		return fmt.Errorf("error binding queue: %s", err)
	}

	return nil
}

func declareQueues(channel *amqp.Channel) error {
	queueBindings := map[string]string{
		"user-created-feed-service-queue":       "user.created",
		"user-deleted-feed-service-queue":       "user.deleted",
		"user-followed-feed-service-queue":      "user.followed",
		"user-unfollowed-feed-service-queue":    "user.unfollowed",
		"user-add-friend-feed-service-queue":    "user.add.friend",
		"user-remove-friend-feed-service-queue": "user.remove.friend",
		"post-created-feed-service-queue":       "post.created",
		"post-updated-feed-service-queue":       "post.updated",
		"post-deleted-feed-service-queue":       "post.deleted",
		"post-liked-feed-service-queue":         "post.liked",
		"post-unliked-feed-service-queue":       "post.unliked",
		"post-viewed-feed-service-queue":        "post.viewed",
	}

	for queueName, routingKey := range queueBindings {
		exchangeName := "post-exchange"
		if strings.Contains(routingKey, "user") {
			exchangeName = "user-exchange"
		}

		if err := declareAndBindQueue(channel, queueName, routingKey, exchangeName); err != nil {
			return err
		}
	}

	return nil
}

func (c *AmqConsumer) handleMessages(deliveries <-chan amqp.Delivery, consume func(delivery amqp.Delivery)) {
	defer func() {
		log.Info().Msg("consume channel is closed")
		c.done <- nil
	}()

	for d := range deliveries {
		consume(d)
	}
}
