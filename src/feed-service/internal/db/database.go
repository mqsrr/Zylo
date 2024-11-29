package db

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"regexp"
	"strings"
	"time"
)

type RecommendationService interface {
	CreateUser(ctx context.Context, userID string) error
	CreatePost(ctx context.Context, postID string, userID string, content string, createdAt time.Time) error
	AddFriend(ctx context.Context, userID, friendID string) error
	RemoveFriend(ctx context.Context, userID, friendID string) error
	FollowUser(ctx context.Context, userID, followedUserID string) error
	UnfollowUser(ctx context.Context, userID, followedID string) error
	LikePost(ctx context.Context, userID, postID string) error
	UnlikePost(ctx context.Context, userID, postID string) error
	ViewPost(ctx context.Context, userID, postID string) error
	UpdatePostTags(ctx context.Context, postID string, content string) error
	DeleteUser(ctx context.Context, userID string) error
	DeletePost(ctx context.Context, postID string) error
	GenerateRecommendedPostIDs(ctx context.Context, userID string, minLikes int, limit int, next *time.Time) ([]string, *time.Time, error)
	Shutdown(ctx context.Context) error
}

type Neo4jStorage struct {
	driver neo4j.DriverWithContext
}

func NewNeo4jStorage(ctx context.Context, config *config.Neo4jConfig) (*Neo4jStorage, error) {
	driver, err := neo4j.NewDriverWithContext(config.Uri, neo4j.BasicAuth(config.Username, config.Password, ""))
	if err != nil {
		return nil, err
	}

	if err = driver.VerifyConnectivity(ctx); err != nil {
		return nil, err
	}

	storage := &Neo4jStorage{
		driver: driver,
	}
	queries := strings.Split(CreateIndexQuery, "CREATE")
	for i := range queries {
		if strings.TrimSpace(queries[i]) == "" {
			continue
		}

		query := "CREATE" + queries[i]
		err := storage.executeWrite(ctx, query, map[string]any{})
		if err != nil {
			return nil, err
		}
	}
	return storage, err
}

func (n *Neo4jStorage) executeWrite(ctx context.Context, query string, params map[string]any) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer session.Close(ctx)

	_, err := session.ExecuteWrite(ctx, func(tx neo4j.ManagedTransaction) (any, error) {
		result, err := tx.Run(ctx, query, params)
		if err != nil {
			return false, err
		}

		summary, err := result.Consume(ctx)
		counters := summary.Counters()

		return counters.RelationshipsCreated() > 0 ||
			counters.RelationshipsDeleted() > 0 ||
			counters.NodesCreated() > 0 ||
			counters.NodesDeleted() > 0, err
	})
	if err != nil {
		return err
	}

	return nil
}

func extractTags(content string) []string {
	re := regexp.MustCompile(`#(\w+)`)

	matches := re.FindAllStringSubmatch(content, -1)

	var tags []string
	for _, match := range matches {
		if len(match) > 1 {
			tags = append(tags, strings.ToLower(match[1]))
		}
	}

	return tags
}

func (n *Neo4jStorage) CreateUser(ctx context.Context, userID string) error {
	return n.executeWrite(ctx, CreateUserQuery, map[string]any{
		"userID": userID,
	})
}

func (n *Neo4jStorage) CreatePost(ctx context.Context, postID, userID string, content string, createdAt time.Time) error {
	tags := extractTags(content)
	return n.executeWrite(ctx, CreatePostQuery, map[string]any{
		"postID":    postID,
		"userID":    userID,
		"createdAt": createdAt.Format(time.RFC3339),
		"tags":      tags,
	})
}

func (n *Neo4jStorage) AddFriend(ctx context.Context, userID, friendID string) error {
	return n.executeWrite(ctx, AddFriendQuery, map[string]any{
		"userID":   userID,
		"friendID": friendID,
	})
}

func (n *Neo4jStorage) RemoveFriend(ctx context.Context, userID, friendID string) error {
	return n.executeWrite(ctx, RemoveFriendQuery, map[string]any{
		"userID":   userID,
		"friendID": friendID,
	})
}

func (n *Neo4jStorage) FollowUser(ctx context.Context, userID, followedID string) error {
	return n.executeWrite(ctx, FollowUserQuery, map[string]any{
		"userID":     userID,
		"followedID": followedID,
	})
}

func (n *Neo4jStorage) UnfollowUser(ctx context.Context, userID, followedID string) error {
	return n.executeWrite(ctx, UnfollowUserQuery, map[string]any{
		"userID":     userID,
		"followedID": followedID,
	})
}

func (n *Neo4jStorage) LikePost(ctx context.Context, userID, postID string) error {
	return n.executeWrite(ctx, UserLikedPostQuery, map[string]any{
		"userID": userID,
		"postID": postID,
	})
}

func (n *Neo4jStorage) UnlikePost(ctx context.Context, userID, postID string) error {
	return n.executeWrite(ctx, UserUnlikedPostQuery, map[string]any{
		"userID": userID,
		"postID": postID,
	})
}

func (n *Neo4jStorage) ViewPost(ctx context.Context, userID, postID string) error {
	return n.executeWrite(ctx, UserViewedPostQuery, map[string]any{
		"userID": userID,
		"postID": postID,
	})
}

func (n *Neo4jStorage) UpdatePostTags(ctx context.Context, postID string, content string) error {
	tags := extractTags(content)
	return n.executeWrite(ctx, UpdatePostQuery, map[string]any{
		"postID": postID,
		"tags":   tags,
	})
}

func (n *Neo4jStorage) DeleteUser(ctx context.Context, userID string) error {
	return n.executeWrite(ctx, DeleteUserQuery, map[string]any{
		"userID": userID,
	})
}

func (n *Neo4jStorage) DeletePost(ctx context.Context, postID string) error {
	return n.executeWrite(ctx, DeletePostQuery, map[string]any{
		"postID": postID,
	})
}

func (n *Neo4jStorage) GenerateRecommendedPostIDs(ctx context.Context, userID string, minLikes int, limit int, next *time.Time) ([]string, *time.Time, error) {
	cursor := time.Now()
	if next != nil {
		cursor = *next
	}

	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeRead})
	defer session.Close(ctx)
	var postIDs []string

	_, err := session.ExecuteRead(ctx, func(tx neo4j.ManagedTransaction) (any, error) {
		result, err := tx.Run(ctx, GenerateRecommendationQuery, map[string]any{
			"userID":   userID,
			"cursor":   cursor.Format(time.RFC3339),
			"limit":    limit,
			"minLikes": minLikes,
		})
		if err != nil {
			return nil, err
		}

		for result.Next(ctx) {
			record := result.Record()
			postIDStr, _ := record.Get("postID")
			createdAtStr, _ := record.Get("createdAt")

			postIDs = append(postIDs, postIDStr.(string))

			createdAt, err := time.Parse(time.RFC3339, createdAtStr.(string))
			if err != nil {
				return nil, err
			}
			next = &createdAt
		}

		return postIDs, result.Err()
	})

	if err != nil {
		return nil, nil, err
	}

	return postIDs, next, nil
}

func (n *Neo4jStorage) Shutdown(ctx context.Context) error {
	return n.driver.Close(ctx)
}
