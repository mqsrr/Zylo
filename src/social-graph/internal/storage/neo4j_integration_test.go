//go:build integration

package storage_test

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"github.com/mqsrr/zylo/social-graph/internal/testutil"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/stretchr/testify/suite"
	"github.com/testcontainers/testcontainers-go/modules/neo4j"
	"testing"
	"time"
)

type Neo4jStorageTestSuite struct {
	suite.Suite
	neo4jContainer *neo4j.Neo4jContainer
	storage        storage.RelationshipStorage
	ctx            context.Context
}

func (suite *Neo4jStorageTestSuite) SetupSuite() {
	neo4jContainer, err := testutil.StartNeo4jContainer()
	suite.Require().NoError(err)

	suite.ctx = context.Background()
	suite.neo4jContainer = neo4jContainer
	neo4jURI, _ := suite.neo4jContainer.BoltUrl(suite.ctx)

	db, err := storage.NewNeo4jStorage(suite.ctx, neo4jURI, "neo4j", "test")
	suite.Require().NoError(err)

	suite.storage = db
}

func (suite *Neo4jStorageTestSuite) TearDownSuite() {
	err := suite.neo4jContainer.Terminate(suite.ctx)
	suite.Require().NoError(err)
}
func (suite *Neo4jStorageTestSuite) TestCreateUser() {
	user := &types.User{
		ID:        ulid.Make(),
		Username:  "testuser",
		Name:      "Test User",
		CreatedAt: time.Now().UTC().Format(time.RFC3339),
	}

	_, err := suite.storage.CreateUser(suite.ctx, user)
	suite.Require().NoError(err)

	createdUser, err := suite.storage.GetUserWithRelationships(suite.ctx, user.ID.String())
	suite.Require().NoError(err)
	suite.Require().EqualValues(user, createdUser.User)
}

func (suite *Neo4jStorageTestSuite) TestGetUserWithRelationships() {
	user := &types.User{
		ID:        ulid.Make(),
		Username:  "testuser",
		Name:      "Test User",
		CreatedAt: time.Now().UTC().Format(time.RFC3339),
	}
	_, err := suite.storage.CreateUser(suite.ctx, user)
	suite.Require().NoError(err)

	userWithRelationships, err := suite.storage.GetUserWithRelationships(suite.ctx, user.ID.String())
	suite.Require().NoError(err)
	suite.Require().EqualValues(user, userWithRelationships.User)
}

func (suite *Neo4jStorageTestSuite) TestFollowUser() {
	user1 := &types.User{ID: ulid.Make(), Username: "user1", Name: "User One", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	user2 := &types.User{ID: ulid.Make(), Username: "user2", Name: "User Two", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, user1)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, user2)
	suite.Require().NoError(err)

	followed, err := suite.storage.FollowUser(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(followed)

	followers, err := suite.storage.GetFollowers(suite.ctx, user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(followers, 1)
	suite.Require().EqualValues(user1, followers[0])
}

func (suite *Neo4jStorageTestSuite) TestGetFollowedPeople() {
	user1 := &types.User{ID: ulid.Make(), Username: "user1", Name: "User One", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	user2 := &types.User{ID: ulid.Make(), Username: "user2", Name: "User Two", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, user1)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, user2)
	suite.Require().NoError(err)

	followed, err := suite.storage.FollowUser(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(followed)

	followers, err := suite.storage.GetFollowedPeople(suite.ctx, user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(followers, 1)
	suite.Require().EqualValues(user2, followers[0])
}

func (suite *Neo4jStorageTestSuite) TestUnfollowUser() {
	user1 := &types.User{ID: ulid.Make(), Username: "user1", Name: "User One", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	user2 := &types.User{ID: ulid.Make(), Username: "user2", Name: "User Two", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, user1)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, user2)
	suite.Require().NoError(err)

	followed, err := suite.storage.FollowUser(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(followed)

	unfollowed, err := suite.storage.UnfollowUser(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(unfollowed)

	followers, err := suite.storage.GetFollowers(suite.ctx, user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(followers, 0)
}

func (suite *Neo4jStorageTestSuite) TestBlockUser() {
	blocker := &types.User{ID: ulid.Make(), Username: "blocker", Name: "Blocker User", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	blocked := &types.User{ID: ulid.Make(), Username: "blocked", Name: "Blocked User", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, blocker)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, blocked)
	suite.Require().NoError(err)

	blockedStatus, err := suite.storage.BlockUser(suite.ctx, blocker.ID.String(), blocked.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(blockedStatus)

	blockedUsers, err := suite.storage.GetBlockedPeople(suite.ctx, blocker.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(blockedUsers, 1)
	suite.Require().EqualValues(blocked, blockedUsers[0])
}

func (suite *Neo4jStorageTestSuite) TestUnblockUser() {
	blocker := &types.User{ID: ulid.Make(), Username: "blocker", Name: "Blocker User", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	blocked := &types.User{ID: ulid.Make(), Username: "blocked", Name: "Blocked User", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, blocker)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, blocked)
	suite.Require().NoError(err)

	blockedStatus, err := suite.storage.BlockUser(suite.ctx, blocker.ID.String(), blocked.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(blockedStatus)

	ok, err := suite.storage.UnblockUser(suite.ctx, blocker.ID.String(), blocked.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(ok)

	blockedUsers, err := suite.storage.GetBlockedPeople(suite.ctx, blocker.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(blockedUsers, 0)
}

func (suite *Neo4jStorageTestSuite) TestUpdateUser() {
	user := &types.User{ID: ulid.Make(), Username: "username", Name: "name", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	updatedUser := &types.User{Bio: "Bio", Name: "UpdatedName", Location: "location"}

	_, err := suite.storage.CreateUser(suite.ctx, user)
	suite.Require().NoError(err)

	ok, err := suite.storage.UpdateUser(suite.ctx, user.ID.String(), updatedUser.Name, updatedUser.Bio, updatedUser.Location)
	suite.Require().NoError(err)
	suite.Require().True(ok)

	userWithRelationships, err := suite.storage.GetUserWithRelationships(suite.ctx, user.ID.String())
	suite.Require().NoError(err)
	suite.Require().Equal(user.ID.String(), userWithRelationships.User.ID.String())
	suite.Require().Equal(updatedUser.Name, userWithRelationships.User.Name)
	suite.Require().Equal(updatedUser.Bio, userWithRelationships.User.Bio)
	suite.Require().Equal(updatedUser.Location, userWithRelationships.User.Location)
}

func (suite *Neo4jStorageTestSuite) TestDeleteById() {
	user := &types.User{ID: ulid.Make(), Username: "username", Name: "name", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, user)
	suite.Require().NoError(err)

	ok, err := suite.storage.DeleteUserByID(suite.ctx, user.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(ok)

	userWithRelationships, err := suite.storage.GetUserWithRelationships(suite.ctx, user.ID.String())
	suite.Require().NoError(err)
	suite.Require().Nil(userWithRelationships)
}

func (suite *Neo4jStorageTestSuite) TestSendAndAcceptFriendRequest() {
	user1 := &types.User{ID: ulid.Make(), Username: "user1", Name: "User One", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	user2 := &types.User{ID: ulid.Make(), Username: "user2", Name: "User Two", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, user1)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, user2)
	suite.Require().NoError(err)

	requestSent, err := suite.storage.SendFriendRequest(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(requestSent)

	pendingRequests, err := suite.storage.GetPendingFriendRequests(suite.ctx, user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(pendingRequests, 1)
	suite.Require().EqualValues(user1, pendingRequests[0])

	userWithRelationship, err := suite.storage.GetUserWithRelationships(suite.ctx, user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().EqualValues(user2, userWithRelationship.SentFriendRequests[0])

	friendAdded, err := suite.storage.AcceptFriendRequest(suite.ctx, user2.ID.String(), user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(friendAdded)

	friends, err := suite.storage.GetFriends(suite.ctx, user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(friends, 1)
	suite.Require().EqualValues(user2, friends[0])
}

func (suite *Neo4jStorageTestSuite) TestDeclineFriendRequest() {
	user1 := &types.User{ID: ulid.Make(), Username: "user1", Name: "User One", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	user2 := &types.User{ID: ulid.Make(), Username: "user2", Name: "User Two", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, user1)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, user2)
	suite.Require().NoError(err)

	requestSent, err := suite.storage.SendFriendRequest(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(requestSent)

	pendingRequests, err := suite.storage.GetPendingFriendRequests(suite.ctx, user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(pendingRequests, 1)
	suite.Require().EqualValues(user1, pendingRequests[0])

	userWithRelationship, err := suite.storage.GetUserWithRelationships(suite.ctx, user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().EqualValues(user2, userWithRelationship.SentFriendRequests[0])

	requestDeclined, err := suite.storage.DeclineFriendRequest(suite.ctx, user2.ID.String(), user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(requestDeclined)

	friends, err := suite.storage.GetFriends(suite.ctx, user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(friends, 0)
}

func (suite *Neo4jStorageTestSuite) TestRemoveFriend() {
	user1 := &types.User{ID: ulid.Make(), Username: "user1", Name: "User One", CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	user2 := &types.User{ID: ulid.Make(), Username: "user2", Name: "User Two", CreatedAt: time.Now().UTC().Format(time.RFC3339)}

	_, err := suite.storage.CreateUser(suite.ctx, user1)
	suite.Require().NoError(err)

	_, err = suite.storage.CreateUser(suite.ctx, user2)
	suite.Require().NoError(err)

	requestSent, err := suite.storage.SendFriendRequest(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(requestSent)

	friendAdded, err := suite.storage.AcceptFriendRequest(suite.ctx, user2.ID.String(), user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(friendAdded)

	removedFriend, err := suite.storage.RemoveFriend(suite.ctx, user1.ID.String(), user2.ID.String())
	suite.Require().NoError(err)
	suite.Require().True(removedFriend)

	friends, err := suite.storage.GetFriends(suite.ctx, user1.ID.String())
	suite.Require().NoError(err)
	suite.Require().Len(friends, 0)
}

func TestNeo4jStorageTestSuite(t *testing.T) {
	suite.Run(t, new(Neo4jStorageTestSuite))
}
