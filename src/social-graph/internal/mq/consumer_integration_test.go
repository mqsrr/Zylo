//go:build integration

package mq_test

import (
	"context"
	"encoding/json"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/mq"
	"github.com/mqsrr/zylo/social-graph/internal/testutil"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"github.com/testcontainers/testcontainers-go/modules/rabbitmq"
	"testing"
	"time"
)

type MQIntegrationTestSuite struct {
	suite.Suite
	rabbitMqContainer *rabbitmq.RabbitMQContainer
	consumer          mq.Consumer
	ctx               context.Context
}

func (suite *MQIntegrationTestSuite) SetupSuite() {
	mqContainer, err := testutil.StartRabbitMqContainer()
	suite.Require().NoError(err, "Failed to start test containers")

	suite.ctx = context.Background()
	suite.rabbitMqContainer = mqContainer

	rabbitmqURI, err := suite.rabbitMqContainer.AmqpURL(suite.ctx)
	suite.Require().NoError(err)

	rabbitMqConfig := &config.RabbitmqConfig{
		AmqpURI:  rabbitmqURI,
		ConnTag:  "test",
		ConnName: "test-conn",
	}

	suite.consumer, err = mq.NewConsumer(rabbitMqConfig)
	suite.Require().NoError(err)
}

func (suite *MQIntegrationTestSuite) TearDownSuite() {
	err := suite.rabbitMqContainer.Terminate(suite.ctx)
	suite.Require().NoError(err)
}

func (suite *MQIntegrationTestSuite) TestPublishAndConsumeMessage() {
	exchangeName := "user-exchange"
	routingKey := "user.created"
	queueName := "user-created-social-graph-queue"

	user := types.UserCreatedMessage{
		ID:       "01F3Z9ZPMDXP1ZB9TT3N4ENMDZ",
		Username: "testuser",
	}
	err := suite.consumer.PublishMessage(exchangeName, routingKey, user)
	require.NoError(suite.T(), err, "Failed to publish message")

	messageReceived := make(chan types.UserCreatedMessage)

	err = suite.consumer.Consume(queueName, func(d amqp.Delivery) {
		var receivedMessage types.UserCreatedMessage
		err := json.Unmarshal(d.Body, &receivedMessage)
		require.NoError(suite.T(), err, "Failed to unmarshal message body")

		messageReceived <- receivedMessage

		err = d.Ack(false)
		require.NoError(suite.T(), err, "Failed to acknowledge message")
	})
	require.NoError(suite.T(), err, "Failed to start consuming messages")

	select {
	case msg := <-messageReceived:
		suite.Require().Equal(user, msg, "Received message does not match the published message")

	case <-time.After(5 * time.Second):
		suite.Fail("Did not receive message in time")
	}
}

func TestMQIntegrationTestSuite(t *testing.T) {
	suite.Run(t, new(MQIntegrationTestSuite))
}
