//go:build unit

package api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	mq "github.com/mqsrr/zylo/social-graph/internal/mq/mocks"
	mocks "github.com/mqsrr/zylo/social-graph/internal/storage/mocks"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/rs/zerolog/log"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/suite"
	"testing"
)

type MessageHandlerTestSuite struct {
	suite.Suite
	ctx          context.Context
	mockStorage  *mocks.MockRelationshipStorage
	mockConsumer *mq.MockConsumer
	server       *Server

	testTypeHandlers map[TestCaseType]func()
}

func (s *MessageHandlerTestSuite) SetupTest() {
	s.ctx = context.Background()
	s.mockStorage = mocks.NewMockRelationshipStorage(s.T())
	s.mockConsumer = mq.NewMockConsumer(s.T())
	s.server = NewServer(nil, s.mockStorage, nil, s.mockConsumer, nil)
}

func (s *MessageHandlerTestSuite) TestHandleUserCreatedMessage_Success() {
	userID := ulid.Make().String()
	userMsg := types.UserCreatedMessage{
		ID:       userID,
		Username: "testuser",
	}
	body, _ := json.Marshal(userMsg)

	s.mockStorage.EXPECT().CreateUser(s.ctx, mock.AnythingOfType("*types.User")).Return(true, nil).Once()
	s.server.handleUserCreatedMessage(amqp.Delivery{Body: body})
}

func (s *MessageHandlerTestSuite) TestHandleUserCreatedMessage_UnmarshalError() {
	var buf bytes.Buffer
	log.Logger = log.Output(&buf)

	s.server.handleUserCreatedMessage(amqp.Delivery{Body: []byte("invalid json")})
	s.mockStorage.AssertNotCalled(s.T(), "CreateUser", mock.Anything, mock.Anything)

	s.Require().Contains(buf.String(), "Could not decode queue message")
}

func (s *MessageHandlerTestSuite) TestHandleUserCreatedMessage_StorageError() {
	userID := ulid.Make().String()
	userMsg := types.UserCreatedMessage{
		ID:       userID,
		Username: "testuser",
	}
	body, _ := json.Marshal(userMsg)
	var buf bytes.Buffer
	log.Logger = log.Output(&buf)

	s.mockStorage.EXPECT().CreateUser(s.ctx, mock.AnythingOfType("*types.User")).Return(false, errors.New("storage err")).Once()
	s.server.handleUserCreatedMessage(amqp.Delivery{Body: body})

	s.Require().Contains(buf.String(), "storage err")
}

func (s *MessageHandlerTestSuite) TestHandleUserUpdatedMessage_Success() {
	userID := ulid.Make().String()
	userMsg := types.UserUpdatedMessage{
		ID:       userID,
		Name:     "Updated Name",
		Bio:      "Updated Bio",
		Location: "Updated Location",
	}
	body, _ := json.Marshal(userMsg)

	s.mockStorage.EXPECT().UpdateUser(mock.Anything, userID, userMsg.Name, userMsg.Bio, userMsg.Location).Return(true, nil).Once()

	s.server.handleUserUpdatedMessage(amqp.Delivery{Body: body})
}

func (s *MessageHandlerTestSuite) TestHandleUserUpdatedMessage_UnmarshalError() {
	var buf bytes.Buffer
	log.Logger = log.Output(&buf)
	s.server.handleUserUpdatedMessage(amqp.Delivery{Body: []byte("invalid json")})

	s.mockStorage.AssertNotCalled(s.T(), "UpdateUser", mock.Anything, mock.Anything, mock.Anything, mock.Anything, mock.Anything)
	s.Require().Contains(buf.String(), "Could not decode queue message")
}

func (s *MessageHandlerTestSuite) TestHandleUserUpdatedMessage_StorageError() {
	userID := ulid.Make().String()
	userMsg := types.UserUpdatedMessage{
		ID:       userID,
		Name:     "Updated Name",
		Bio:      "Updated Bio",
		Location: "Updated Location",
	}
	body, _ := json.Marshal(userMsg)

	var buf bytes.Buffer
	log.Logger = log.Output(&buf)

	s.mockStorage.EXPECT().UpdateUser(mock.Anything, userID, userMsg.Name, userMsg.Bio, userMsg.Location).Return(false, errors.New("storage error")).Once()

	s.server.handleUserUpdatedMessage(amqp.Delivery{Body: body})
	s.Require().Contains(buf.String(), "storage error")
}

func (s *MessageHandlerTestSuite) TestHandleUserDeletedMessage_Success() {
	userID := ulid.Make().String()
	userMsg := types.UserDeletedMessage{
		ID: userID,
	}
	body, _ := json.Marshal(userMsg)

	s.mockStorage.EXPECT().DeleteUserByID(s.ctx, userID).Return(true, nil).Once()
	s.server.handleUserDeletedMessage(amqp.Delivery{Body: body})
}

func (s *MessageHandlerTestSuite) TestHandleUserDeletedMessage_UnmarshalError() {
	var buf bytes.Buffer
	log.Logger = log.Output(&buf)
	s.server.handleUserDeletedMessage(amqp.Delivery{Body: []byte("invalid json")})

	s.mockStorage.AssertNotCalled(s.T(), "DeleteUserByID", mock.Anything, mock.Anything)
	s.Require().Contains(buf.String(), "Could not decode queue message")
}

func (s *MessageHandlerTestSuite) TestHandleUserDeletedMessage_StorageError() {
	userID := ulid.Make().String()
	userMsg := types.UserDeletedMessage{
		ID: userID,
	}
	body, _ := json.Marshal(userMsg)
	var buf bytes.Buffer
	log.Logger = log.Output(&buf)

	s.mockStorage.EXPECT().DeleteUserByID(s.ctx, userID).Return(false, errors.New("storage error")).Once()

	s.server.handleUserDeletedMessage(amqp.Delivery{Body: body})

	s.Require().Contains(buf.String(), "storage error")
}

func TestMessageHandlerTestSuite(t *testing.T) {
	suite.Run(t, new(MessageHandlerTestSuite))
}
