package storage

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j/dbtype"
	"github.com/oklog/ulid/v2"
)

type Storage interface {
	CreateUser(ctx context.Context, user *types.User) (bool, error)
	UpdateUser(ctx context.Context, userID, name, bio, location string) (bool, error)
	DeleteUserByID(ctx context.Context, userId string) (bool, error)
}

type RelationshipStorage interface {
	Storage
	GetUserWithRelationships(ctx context.Context, userID string) (*types.UserWithRelationships, error)
	GetFollowers(ctx context.Context, userID string) ([]*types.User, error)
	GetFollowedPeople(ctx context.Context, userID string) ([]*types.User, error)
	GetBlockedPeople(ctx context.Context, userID string) ([]*types.User, error)
	GetFriends(ctx context.Context, userID string) ([]*types.User, error)
	GetPendingFriendRequests(ctx context.Context, userID string) ([]*types.User, error)

	RemoveFriend(ctx context.Context, userID, friendID string) (bool, error)
	FollowUser(ctx context.Context, followerId, followedId string) (bool, error)
	UnfollowUser(ctx context.Context, followerId, followedId string) (bool, error)
	SendFriendRequest(ctx context.Context, userID, receiverID string) (bool, error)
	AcceptFriendRequest(ctx context.Context, userID, receiverID string) (bool, error)
	DeclineFriendRequest(ctx context.Context, userID, receiverID string) (bool, error)
	BlockUser(ctx context.Context, blockerID, blockedID string) (bool, error)
	UnblockUser(ctx context.Context, blockerID, blockedID string) (bool, error)
}

type Neo4jStorage struct {
	driver neo4j.DriverWithContext
}

func (n *Neo4jStorage) executeRead(ctx context.Context, query string, params map[string]any, resultHandler func(record dbtype.Node) *types.User) ([]*types.User, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeRead})
	defer session.Close(ctx)

	results, err := session.ExecuteRead(ctx, func(tx neo4j.ManagedTransaction) (any, error) {
		var users []*types.User
		result, err := tx.Run(ctx, query, params)

		for result.Next(ctx) {
			record := result.Record().AsMap()["u2"].(dbtype.Node)
			users = append(users, resultHandler(record))
		}
		return users, err
	})

	return results.([]*types.User), err
}

func (n *Neo4jStorage) executeWrite(ctx context.Context, query string, params map[string]any) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	result, err := session.ExecuteWrite(ctx, func(tx neo4j.ManagedTransaction) (any, error) {
		result, err := tx.Run(ctx, query, params)
		if err != nil {
			return false, err
		}

		summary, err := result.Consume(ctx)
		counters := summary.Counters()

		return counters.RelationshipsCreated() > 0 ||
			counters.RelationshipsDeleted() > 0 ||
			counters.NodesCreated() > 0 ||
			counters.ContainsUpdates() ||
			counters.NodesDeleted() > 0, err
	})
	if err != nil {
		return false, err
	}

	return result.(bool), nil
}

func mapUser(record dbtype.Node) *types.User {
	p := record.Props

	getStringOrNil := func(value interface{}) string {
		str, ok := value.(string)
		if ok && str != "" {
			return str
		}
		return ""
	}
	return &types.User{
		ID:        ulid.MustParse(p["id"].(string)),
		Username:  p["username"].(string),
		Name:      p["name"].(string),
		Bio:       getStringOrNil(p["bio"]),
		Location:  getStringOrNil(p["location"]),
		CreatedAt: p["created_at"].(string),
	}
}

func mapUsers(records []interface{}) []*types.User {
	var users []*types.User
	for _, record := range records {
		user := mapUser(record.(dbtype.Node))
		users = append(users, user)
	}
	return users
}

func NewNeo4jStorage(ctx context.Context, uri, username, password string) (*Neo4jStorage, error) {
	driver, err := neo4j.NewDriverWithContext(uri, neo4j.BasicAuth(username, password, ""))
	if err != nil {
		return nil, err
	}

	if err = driver.VerifyConnectivity(ctx); err != nil {
		return nil, err
	}

	storage := &Neo4jStorage{
		driver: driver,
	}

	_, err = storage.executeWrite(ctx, CreateIndexQuery, map[string]any{})
	return storage, err
}

func (n *Neo4jStorage) GetUserWithRelationships(ctx context.Context, userID string) (*types.UserWithRelationships, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeRead})
	defer session.Close(ctx)

	user, err := session.ExecuteRead(ctx, func(tx neo4j.ManagedTransaction) (any, error) {
		var user *types.UserWithRelationships
		result, err := tx.Run(ctx, GetUserWithRelationshipQuery, map[string]any{
			"userID": userID,
		})

		if hasNext := result.Next(ctx); hasNext != true {
			return nil, nil
		}

		record := result.Record().AsMap()
		if err != nil {
			return nil, err
		}
		user = &types.UserWithRelationships{
			User:                   mapUser(record["u"].(dbtype.Node)),
			Followers:              mapUsers(record["followers"].([]interface{})),
			FollowedPeople:         mapUsers(record["followedPeople"].([]interface{})),
			BlockedPeople:          mapUsers(record["blockedPeople"].([]interface{})),
			Friends:                mapUsers(record["friends"].([]interface{})),
			SentFriendRequests:     mapUsers(record["sentFriendRequests"].([]interface{})),
			ReceivedFriendRequests: mapUsers(record["receivedFriendRequests"].([]interface{})),
		}

		return user, err
	})

	if user == nil {
		return nil, err
	}

	return user.(*types.UserWithRelationships), err
}

func (n *Neo4jStorage) GetFollowers(ctx context.Context, userID string) ([]*types.User, error) {
	return n.executeRead(ctx, GetFollowersQuery, map[string]any{"userID": userID}, mapUser)
}

func (n *Neo4jStorage) GetFollowedPeople(ctx context.Context, userID string) ([]*types.User, error) {
	return n.executeRead(ctx, GetFollowedPeopleQuery, map[string]any{"userID": userID}, mapUser)
}

func (n *Neo4jStorage) GetBlockedPeople(ctx context.Context, userID string) ([]*types.User, error) {
	return n.executeRead(ctx, GetBlockedPeopleQuery, map[string]any{"userID": userID}, mapUser)
}

func (n *Neo4jStorage) GetFriends(ctx context.Context, userID string) ([]*types.User, error) {
	return n.executeRead(ctx, GetFriendsQuery, map[string]any{"userID": userID}, mapUser)
}

func (n *Neo4jStorage) GetPendingFriendRequests(ctx context.Context, userID string) ([]*types.User, error) {
	return n.executeRead(ctx, GetPendingFriendRequestsQuery, map[string]any{"userID": userID}, mapUser)
}

func (n *Neo4jStorage) CreateUser(ctx context.Context, user *types.User) (bool, error) {
	return n.executeWrite(ctx, CreateUserQuery, map[string]any{
		"id":         user.ID.String(),
		"username":   user.Username,
		"name":       user.Name,
		"created_at": user.CreatedAt,
	})
}

func (n *Neo4jStorage) UpdateUser(ctx context.Context, userID, name, bio, location string) (bool, error) {
	return n.executeWrite(ctx, UpdateUserQuery, map[string]any{
		"id":       userID,
		"name":     name,
		"bio":      bio,
		"location": location,
	})
}

func (n *Neo4jStorage) DeleteUserByID(ctx context.Context, userID string) (bool, error) {
	return n.executeWrite(ctx, DeleteUserByIDQuery, map[string]any{"userID": userID})
}

func (n *Neo4jStorage) FollowUser(ctx context.Context, followerID string, followedID string) (bool, error) {
	return n.executeWrite(ctx, FollowUserQuery, map[string]any{
		"followerID": followerID,
		"followedID": followedID,
	})
}

func (n *Neo4jStorage) UnfollowUser(ctx context.Context, followerID string, followedID string) (bool, error) {
	return n.executeWrite(ctx, UnfollowUserQuery, map[string]any{
		"followerID": followerID,
		"followedID": followedID,
	})
}

func (n *Neo4jStorage) SendFriendRequest(ctx context.Context, userID string, receiverID string) (bool, error) {
	return n.executeWrite(ctx, SendFriendRequestQuery, map[string]any{
		"userID":     userID,
		"receiverID": receiverID,
	})
}

func (n *Neo4jStorage) AcceptFriendRequest(ctx context.Context, userID string, receiverID string) (bool, error) {
	return n.executeWrite(ctx, AcceptFriendRequestQuery, map[string]any{
		"userID":     userID,
		"receiverID": receiverID,
	})
}

func (n *Neo4jStorage) DeclineFriendRequest(ctx context.Context, userID string, receiverID string) (bool, error) {
	return n.executeWrite(ctx, DeclineFriendRequestQuery, map[string]any{
		"userID":     userID,
		"receiverID": receiverID,
	})
}

func (n *Neo4jStorage) RemoveFriend(ctx context.Context, userID string, friendID string) (bool, error) {
	return n.executeWrite(ctx, RemoveFriendQuery, map[string]any{
		"userID":   userID,
		"friendID": friendID,
	})
}

func (n *Neo4jStorage) BlockUser(ctx context.Context, blockerID string, blockedID string) (bool, error) {
	return n.executeWrite(ctx, BlockUserQuery, map[string]any{
		"blockerID":     blockerID,
		"blockedUserID": blockedID,
	})
}

func (n *Neo4jStorage) UnblockUser(ctx context.Context, blockerID string, blockedID string) (bool, error) {
	return n.executeWrite(ctx, UnblockUserQuery, map[string]any{
		"blockerID":     blockerID,
		"blockedUserID": blockedID,
	})
}
