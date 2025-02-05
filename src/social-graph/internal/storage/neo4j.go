package storage

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j/dbtype"
	"github.com/oklog/ulid/v2"
	"time"
)

type Storage interface {
	CreateUser(ctx context.Context, userID ulid.ULID) (bool, error)
	DeleteUserByID(ctx context.Context, userId ulid.ULID) (bool, error)
}

type RelationshipStorage interface {
	Storage
	GetUserWithRelationships(ctx context.Context, userID ulid.ULID) (*types.UserWithRelationships, error)
	BatchGetUserWithRelationships(ctx context.Context, userIDs []ulid.ULID) (map[ulid.ULID]*types.UserWithRelationships, error)
	GetFollowers(ctx context.Context, userID ulid.ULID) (*types.Relationship, error)
	GetFollowedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error)
	GetBlockedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error)
	GetFriends(ctx context.Context, userID ulid.ULID) (*types.Relationship, error)
	GetPendingFriendRequests(ctx context.Context, userID ulid.ULID) (*types.Relationship, error)

	RemoveFriend(ctx context.Context, userID, friendID ulid.ULID) (bool, error)
	FollowUser(ctx context.Context, followerId, followedId ulid.ULID) (bool, error)
	UnfollowUser(ctx context.Context, followerId, followedId ulid.ULID) (bool, error)
	SendFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error)
	AcceptFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error)
	DeclineFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error)
	BlockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error)
	UnblockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error)
}

type Neo4jStorage struct {
	driver neo4j.DriverWithContext
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

	session := driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	_, err = session.Run(ctx,
		`CREATE INDEX node_range_index_id IF NOT EXISTS FOR (u:User) ON (u.id)`,
		nil)
	return storage, err
}

func (n *Neo4jStorage) CreateUser(ctx context.Context, userID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	createdAt := time.Now().Format(time.RFC3339)
	_, err := session.Run(ctx, `MERGE (u:User {id: $id}) SET u.createdAt = $createdAt`, map[string]interface{}{
		"id":        userID.String(),
		"createdAt": createdAt,
	})

	if err != nil {
		return false, err
	}

	return true, nil
}

func (n *Neo4jStorage) DeleteUserByID(ctx context.Context, userID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	_, err := session.Run(ctx,
		`
		MATCH (u:User {id: $id})
		DETACH DELETE u`,
		map[string]interface{}{
			"id": userID.String(),
		})

	if err != nil {
		return false, err
	}

	return true, nil
}

func (n *Neo4jStorage) GetUserWithRelationships(ctx context.Context, userID ulid.ULID) (*types.UserWithRelationships, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeRead})
	defer session.Close(ctx)

	result, err := session.Run(ctx,
		`
			MATCH (u:User {id: $id})
				OPTIONAL MATCH (follower:User)-[r1:FOLLOWS]->(u)
				OPTIONAL MATCH (u)-[r2:FOLLOWS]->(followed:User)
				OPTIONAL MATCH (u)-[r3:BLOCKED]->(blocked:User)
				OPTIONAL MATCH (u)-[r4:FRIEND]-(friend:User)
				OPTIONAL MATCH (u)-[r5:FRIEND_REQUEST]->(pendingRequestReceiver:User)
				OPTIONAL MATCH (pendingRequestSender:User)-[r6:FRIEND_REQUEST]->(u)
			RETURN 
				COLLECT(DISTINCT { id: follower.id, createdAt: r1.createdAt }) as followers,
				COLLECT(DISTINCT  { id: followed.id, createdAt: r2.createdAt }) as following,
				COLLECT(DISTINCT { id: blocked.id, createdAt: r3.createdAt }) as blocks,
				COLLECT(DISTINCT { id: friend.id, createdAt: r4.createdAt }) as friends,
				COLLECT(DISTINCT { id: pendingRequestReceiver.id, createdAt: r5.createdAt }) as sentFriendRequests,
				COLLECT(DISTINCT { id: pendingRequestSender.id, createdAt: r6.createdAt }) as receivedFriendRequests`,
		map[string]interface{}{
			"id": userID.String(),
		})

	if err != nil {
		return nil, err
	}

	var uwr types.UserWithRelationships
	uwr.UserID = userID
	for result.Next(ctx) {
		record := result.Record()

		extractRelationship := func(key string) (*types.Relationship, error) {
			rawList, found := record.Get(key)
			if !found {
				return &types.Relationship{
					IDs:       []ulid.ULID{},
					CreatedAt: map[ulid.ULID]string{},
				}, nil
			}
			list, ok := rawList.([]interface{})
			if !ok {
				return nil, nil
			}

			var ids []ulid.ULID
			createdAt := make(map[ulid.ULID]string)
			for _, item := range list {
				props := item.(map[string]interface{})

				idStr, _ := props["id"].(string)
				ts, _ := props["createdAt"].(string)
				if idStr == "" {
					continue
				}
				id, err := ulid.Parse(idStr)
				if err != nil {
					continue
				}
				ids = append(ids, id)
				createdAt[id] = ts
			}
			return &types.Relationship{
				IDs:       ids,
				CreatedAt: createdAt,
			}, nil
		}

		if rel, err := extractRelationship("followers"); err == nil {
			uwr.Follows = &types.FollowRequests{Followers: rel}
		}
		if rel, err := extractRelationship("following"); err == nil {
			if uwr.Follows == nil {
				uwr.Follows = &types.FollowRequests{}
			}
			uwr.Follows.Following = rel
		}
		if rel, err := extractRelationship("blocks"); err == nil {
			uwr.Blocks = rel
		}
		if rel, err := extractRelationship("friends"); err == nil {
			uwr.Friends = rel
		}
		if rel, err := extractRelationship("sentFriendRequests"); err == nil {
			uwr.FriendRequests = &types.FriendRequests{Sent: rel}
		}
		if rel, err := extractRelationship("receivedFriendRequests"); err == nil {
			if uwr.FriendRequests == nil {
				uwr.FriendRequests = &types.FriendRequests{}
			}
			uwr.FriendRequests.Received = rel
		}
	}
	if err = result.Err(); err != nil {
		return nil, err
	}
	return &uwr, nil
}
func (n *Neo4jStorage) BatchGetUserWithRelationships(ctx context.Context, userIDs []ulid.ULID) (map[ulid.ULID]*types.UserWithRelationships, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeRead})
	defer session.Close(ctx)

	idsStr := make([]string, 0, len(userIDs))
	for _, id := range userIDs {
		idsStr = append(idsStr, id.String())
	}

	result, err := session.Run(ctx,
		`
			UNWIND $ids AS uid
			MATCH (u:User {id: uid})
			  OPTIONAL MATCH (follower:User)-[r1:FOLLOWS]->(u)
			  OPTIONAL MATCH (u)-[r2:FOLLOWS]->(followed:User)
			  OPTIONAL MATCH (u)-[r3:BLOCKED]->(blocked:User)
			  OPTIONAL MATCH (u)-[r4:FRIEND]-(friend:User)
			  OPTIONAL MATCH (u)-[r5:FRIEND_REQUEST]->(pendingRequestReceiver:User)
			  OPTIONAL MATCH (pendingRequestSender:User)-[r6:FRIEND_REQUEST]->(u)
			RETURN 
			  u.id AS userId,
			  COLLECT(DISTINCT { id: follower.id, createdAt: r1.createdAt }) AS followers,
			  COLLECT(DISTINCT { id: followed.id, createdAt: r2.createdAt }) AS following,
			  COLLECT(DISTINCT { id: blocked.id, createdAt: r3.createdAt }) AS blocks,
			  COLLECT(DISTINCT { id: friend.id, createdAt: r4.createdAt }) AS friends,
			  COLLECT(DISTINCT { id: pendingRequestReceiver.id, createdAt: r5.createdAt }) AS sentFriendRequests,
			  COLLECT(DISTINCT { id: pendingRequestSender.id, createdAt: r6.createdAt }) AS receivedFriendRequests
		`,
		map[string]interface{}{
			"ids": idsStr,
		})
	if err != nil {
		return nil, types.NewInternalError("Unexpected Error", err)
	}

	usersMap := make(map[ulid.ULID]*types.UserWithRelationships)

	extractRelationship := func(record *neo4j.Record, key string) (*types.Relationship, error) {
		rawList, found := record.Get(key)
		if !found {
			return &types.Relationship{
				IDs:       []ulid.ULID{},
				CreatedAt: map[ulid.ULID]string{},
			}, nil
		}
		list, ok := rawList.([]interface{})
		if !ok {
			return &types.Relationship{
				IDs:       []ulid.ULID{},
				CreatedAt: map[ulid.ULID]string{},
			}, nil
		}
		var ids []ulid.ULID
		createdAt := make(map[ulid.ULID]string)
		for _, item := range list {
			props, ok := item.(map[string]interface{})
			if !ok {
				continue
			}
			idStr, _ := props["id"].(string)
			ts, _ := props["createdAt"].(string)
			if idStr == "" {
				continue
			}
			parsedID, err := ulid.Parse(idStr)
			if err != nil {
				continue
			}
			ids = append(ids, parsedID)
			createdAt[parsedID] = ts
		}
		return &types.Relationship{
			IDs:       ids,
			CreatedAt: createdAt,
		}, nil
	}

	for result.Next(ctx) {
		record := result.Record()

		userIDStr, ok := record.Get("userId")
		if !ok || userIDStr == "" {
			continue
		}
		userID, err := ulid.Parse(userIDStr.(string))
		if err != nil {
			continue
		}

		uwr := &types.UserWithRelationships{
			UserID: userID,
		}

		if rel, err := extractRelationship(record, "followers"); err == nil {
			uwr.Follows = &types.FollowRequests{Followers: rel}
		}
		if rel, err := extractRelationship(record, "following"); err == nil {
			if uwr.Follows == nil {
				uwr.Follows = &types.FollowRequests{}
			}
			uwr.Follows.Following = rel
		}
		if rel, err := extractRelationship(record, "blocks"); err == nil {
			uwr.Blocks = rel
		}
		if rel, err := extractRelationship(record, "friends"); err == nil {
			uwr.Friends = rel
		}
		if rel, err := extractRelationship(record, "sentFriendRequests"); err == nil {
			uwr.FriendRequests = &types.FriendRequests{Sent: rel}
		}
		if rel, err := extractRelationship(record, "receivedFriendRequests"); err == nil {
			if uwr.FriendRequests == nil {
				uwr.FriendRequests = &types.FriendRequests{}
			}
			uwr.FriendRequests.Received = rel
		}

		usersMap[userID] = uwr
	}

	if err = result.Err(); err != nil {
		return nil, err
	}

	return usersMap, nil
}

func (n *Neo4jStorage) getRelationship(ctx context.Context, query string, params map[string]interface{}) (*types.Relationship, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeRead})
	defer session.Close(ctx)

	result, err := session.Run(ctx, query, params)
	if err != nil {
		return nil, err
	}

	var ids []ulid.ULID
	createdAt := make(map[ulid.ULID]string)
	for result.Next(ctx) {
		record := result.Record().AsMap()["u2"].(dbtype.Node)
		props := record.Props
		idStr := props["id"].(string)
		ts := props["createdAt"].(string)
		if idStr == "" {
			continue
		}
		id, err := ulid.Parse(idStr)
		if err != nil {
			continue
		}
		ids = append(ids, id)
		createdAt[id] = ts
	}
	if err = result.Err(); err != nil {
		return nil, types.NewInternalError("Unexpected Error", err)
	}
	return &types.Relationship{
		IDs:       ids,
		CreatedAt: createdAt,
	}, nil
}

func (n *Neo4jStorage) GetFollowers(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	return n.getRelationship(ctx,
		`
			MATCH (u1:User {id: $id})<-[r:FOLLOWS]-(u2:User)
			RETURN u2`,
		map[string]interface{}{
			"id": userID.String(),
		})
}

func (n *Neo4jStorage) GetFollowedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	return n.getRelationship(ctx,
		`
			MATCH (u1:User {id: $id})-[r:FOLLOWS]->(u2:User)
			RETURN u2`,
		map[string]interface{}{
			"id": userID.String(),
		})
}

func (n *Neo4jStorage) GetBlockedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	return n.getRelationship(ctx,
		`
			MATCH (u1:User {id: $id})-[r:BLOCKED]->(u2:User)
			RETURN u2`,
		map[string]interface{}{
			"id": userID.String(),
		})
}

func (n *Neo4jStorage) GetFriends(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	return n.getRelationship(ctx,
		`
			MATCH (u1:User {id: $id})-[r:FRIEND]-(u2:User)
			RETURN DISTINCT u2`,
		map[string]interface{}{
			"id": userID.String(),
		})
}

func (n *Neo4jStorage) GetPendingFriendRequests(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	return n.getRelationship(ctx, `
			MATCH (u1:User {id: $id})<-[r:FRIEND_REQUEST]-(u2:User)
			RETURN u2`,
		map[string]interface{}{
			"id": userID.String(),
		})
}

func (n *Neo4jStorage) RemoveFriend(ctx context.Context, userID, friendID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	result, err := session.Run(ctx, `
			MATCH (u1:User {id: $id})-[r:FRIEND]-(u2:User {id: $friendID})
			DELETE r`,
		map[string]interface{}{
			"id":       userID.String(),
			"friendID": friendID.String(),
		})
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsDeleted() > 0, nil
}

func (n *Neo4jStorage) FollowUser(ctx context.Context, followerID, followedID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	createdAt := time.Now().Format(time.RFC3339)
	result, err := session.Run(ctx, `
			MATCH (u1:User {id: $followerID}), (u2:User {id: $followedID})
			WHERE NOT EXISTS ((u1)-[:FOLLOWS]->(u2))
			CREATE (u1)-[:FOLLOWS {createdAt: $createdAt}]->(u2)`,
		map[string]interface{}{
			"followerID": followerID.String(),
			"followedID": followedID.String(),
			"createdAt":  createdAt,
		})

	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsCreated() > 0, nil
}

func (n *Neo4jStorage) UnfollowUser(ctx context.Context, followerID, followedID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	result, err := session.Run(ctx, `
			MATCH (follower:User {id: $followerID})-[r:FOLLOWS]->(followed:User {id: $followedID})
			DELETE r`,
		map[string]interface{}{
			"followerID": followerID.String(),
			"followedID": followedID.String(),
		})
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsDeleted() > 0, nil
}

func (n *Neo4jStorage) SendFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	createdAt := time.Now().Format(time.RFC3339)
	result, err := session.Run(ctx, `
			MATCH (u1:User {id: $id}), (u2:User {id: $receiverID})
			WHERE NOT EXISTS ((u1)-[:FRIEND]-(u2)) AND NOT EXISTS ((u1)-[:FRIEND_REQUEST]-(u2))
			CREATE (u1)-[:FRIEND_REQUEST {createdAt: $createdAt}]->(u2)
			RETURN u1, u2`,
		map[string]interface{}{
			"id":         userID.String(),
			"receiverID": receiverID.String(),
			"createdAt":  createdAt,
		})
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsCreated() > 0, nil
}

func (n *Neo4jStorage) AcceptFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	createdAt := time.Now().Format(time.RFC3339)
	result, err := session.Run(ctx, `
			MATCH (sender:User {id: $id})<-[r:FRIEND_REQUEST]-(receiver:User {id: $receiverID})
			DELETE r
			CREATE (sender)-[:FRIEND {createdAt: $createdAt}]->(receiver),
				   (receiver)-[:FRIEND {createdAt: $createdAt}]->(sender)`,
		map[string]interface{}{
			"id":         userID.String(),
			"receiverID": receiverID.String(),
			"createdAt":  createdAt,
		})
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsCreated() > 0, nil
}

func (n *Neo4jStorage) DeclineFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	result, err := session.Run(ctx, `
			MATCH (sender:User {id: $id})<-[r:FRIEND_REQUEST]-(receiver:User {id: $receiverID})
			DELETE r`,
		map[string]interface{}{
			"userID":     userID.String(),
			"receiverID": receiverID.String(),
		})
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsDeleted() > 0, nil
}

func (n *Neo4jStorage) BlockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	createdAt := time.Now().Format(time.RFC3339)
	result, err := session.Run(ctx, `
			MATCH (u1:User {id: $blockerID}), (u2:User {id: $blockedUserID})
			OPTIONAL MATCH (u1)-[r]-(u2)
			DELETE r
			CREATE (u1)-[:BLOCKED {createdAt: $createdAt}]->(u2)`,
		map[string]interface{}{
			"blockerID":     blockerID.String(),
			"blockedUserID": blockedID.String(),
			"createdAt":     createdAt,
		})
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsCreated() > 0, nil
}

func (n *Neo4jStorage) UnblockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	result, err := session.Run(ctx, `
			MATCH (u1:User {id: $blockerID})-[r:BLOCKED]->(u2:User {id: $blockedUserID})
			DELETE r`,
		map[string]interface{}{
			"blockerID":     blockerID.String(),
			"blockedUserID": blockedID.String(),
		})

	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	summary, err := result.Consume(ctx)
	if err != nil {
		return false, types.NewInternalError("Unexpected Error", err)
	}

	return summary.Counters().RelationshipsDeleted() > 0, nil
}
