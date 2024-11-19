//go:build integration

package api_test

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/testutil"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/stretchr/testify/suite"
	"testing"
	"time"
)

type MessageHandlerIntegrationSuite struct {
	suite.Suite
}

func (suite *MessageHandlerIntegrationSuite) TestHandleUserCreatedMessage() {
	userID := ulid.Make()
	userMsg := types.UserCreatedMessage{
		ID:       userID.String(),
		Username: "testuser",
	}

	err := testutil.MqConsumer.PublishMessage("user-exchange", "user.created", userMsg)
	suite.Require().NoError(err)

	time.Sleep(500 * time.Millisecond)

	user, err := testutil.RelationshipStorage.GetUserWithRelationships(context.Background(), userID.String())
	suite.Require().NoError(err)
	suite.Equal("testuser", user.User.Username)
}

func (suite *MessageHandlerIntegrationSuite) TestHandleUserUpdatedMessage() {
	userID := ulid.Make()
	userMsg := types.UserCreatedMessage{
		ID:       userID.String(),
		Username: "initialuser",
	}

	err := testutil.MqConsumer.PublishMessage("user-exchange", "user.created", userMsg)
	suite.Require().NoError(err)

	updateMsg := types.UserUpdatedMessage{
		ID:       userID.String(),
		Name:     "updateduser",
		Bio:      "updatedbio",
		Location: "updatedlocation",
	}

	err = testutil.MqConsumer.PublishMessage("user-exchange", "user.updated", updateMsg)
	suite.Require().NoError(err)

	time.Sleep(500 * time.Millisecond)

	user, err := testutil.RelationshipStorage.GetUserWithRelationships(context.Background(), userID.String())
	suite.Require().NoError(err)
	suite.Equal("updateduser", user.User.Name)
	suite.Equal("updatedbio", user.User.Bio)
	suite.Equal("updatedlocation", user.User.Location)
}

func (suite *MessageHandlerIntegrationSuite) TestHandleUserDeletedMessage() {
	userID := ulid.Make()
	userMsg := types.UserCreatedMessage{
		ID:       userID.String(),
		Username: "deletetestuser",
	}

	err := testutil.MqConsumer.PublishMessage("user-exchange", "user.created", userMsg)
	suite.Require().NoError(err)

	deleteMsg := types.UserDeletedMessage{ID: userID.String()}
	err = testutil.MqConsumer.PublishMessage("user-exchange", "user.deleted", deleteMsg)
	suite.Require().NoError(err)

	time.Sleep(1000 * time.Millisecond)

	user, err := testutil.RelationshipStorage.GetUserWithRelationships(context.Background(), userID.String())
	suite.NoError(err)
	suite.Nil(user)
}

func TestMessageHandlerIntegrationSuite(t *testing.T) {
	suite.Run(t, new(MessageHandlerIntegrationSuite))
}
