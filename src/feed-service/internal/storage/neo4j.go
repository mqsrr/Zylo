package storage

import (
	"context"
	"fmt"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/proto/github.com/mqsrr/zylo/feed-service/proto"
	"github.com/mqsrr/zylo/feed-service/internal/types"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"github.com/rs/zerolog/log"
	"regexp"
	"strings"
	"time"
)

func NewNeo4jDriverWithContext(ctx context.Context, config *config.Neo4j) (neo4j.DriverWithContext, error) {
	driver, err := neo4j.NewDriverWithContext(config.Uri, neo4j.BasicAuth(config.Username, config.Password, ""))
	if err != nil {
		return nil, err
	}

	if err = driver.VerifyConnectivity(ctx); err != nil {
		return nil, err
	}

	return driver, nil
}

type UserRepository interface {
	CreateUser(ctx context.Context, userID string, createdAt time.Time) error
	DeleteUser(ctx context.Context, userID string) error
}

type Neo4jUserRespository struct {
	driver neo4j.DriverWithContext
}

func NewNeo4jUserRepository(ctx context.Context, driver neo4j.DriverWithContext) (*Neo4jUserRespository, error) {
	storage := &Neo4jUserRespository{
		driver: driver,
	}

	session := driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	if _, err := session.Run(ctx,
		`CREATE INDEX user_id_index IF NOT EXISTS FOR (u:User) ON (u.id);`,
		nil); err != nil {
		return nil, err
	}

	return storage, nil
}

func (n *Neo4jUserRespository) CreateUser(ctx context.Context, userID string, createdAt time.Time) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `MERGE (u:User {id: $id}) SET u.createdAt = $createdAt`, map[string]interface{}{
		"id":        userID,
		"createdAt": createdAt,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().NodesCreated() < 0 {
		return types.NewBadRequest(fmt.Sprintf("User with id %s already exists", userID))
	}

	return nil
}

func (n *Neo4jUserRespository) DeleteUser(ctx context.Context, userID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx,
		`
		MATCH (u:User {id: $id})
		DETACH DELETE u`,
		map[string]interface{}{
			"id": userID,
		})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().NodesDeleted() < 0 {
		return types.NewNotFound(fmt.Sprintf("User with id %s does not exists", userID))
	}

	return nil
}

type PostRepository interface {
	CreatePost(ctx context.Context, postID string, userID string, content string, createdAt time.Time) error
	UpdatePostTags(ctx context.Context, postID string, content string) error
	DeletePost(ctx context.Context, postID string) error
}

type Neo4jPostRepository struct {
	driver neo4j.DriverWithContext
}

func NewNeo4jPostRepository(ctx context.Context, driver neo4j.DriverWithContext) (*Neo4jPostRepository, error) {
	storage := &Neo4jPostRepository{
		driver: driver,
	}

	session := driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	statements := []string{
		"CREATE INDEX post_id_index IF NOT EXISTS FOR (p:Post) ON (p.id)",
		"CREATE INDEX post_created_at_index IF NOT EXISTS FOR (p:Post) ON (p.createdAt)",
		"CREATE INDEX post_tags_index IF NOT EXISTS FOR (p:Post) ON (p.tags)",
	}

	for _, stmt := range statements {
		if _, err := session.Run(ctx, stmt, nil); err != nil {
			return nil, err
		}
	}
	return storage, nil
}

func (n *Neo4jPostRepository) CreatePost(ctx context.Context, postID string, userID string, content string, createdAt time.Time) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	tags := extractTags(content)

	result, err := session.Run(ctx, `
		MATCH (u:User {id: $userID})
		MERGE (u)-[:CREATED]->(p:Post {id: $id, createdAt: $createdAt, tags: $tags, likes: 0, views: 0})`, map[string]interface{}{
		"id":        postID,
		"userID":    userID,
		"createdAt": createdAt,
		"tags":      tags,
	})
	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().NodesCreated() < 0 {
		return types.NewBadRequest(fmt.Sprintf("Post with id %s already exists", postID))
	}

	return nil
}

func (n *Neo4jPostRepository) UpdatePostTags(ctx context.Context, postID string, content string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	tags := extractTags(content)

	result, err := session.Run(ctx, `
		MATCH (p:Post {id: $id})
		SET p.tags = $tags`, map[string]interface{}{
		"id":   postID,
		"tags": tags,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if !resultSummary.Counters().ContainsUpdates() {
		return types.NewNotFound(fmt.Sprintf("Post with id %s does not exist", postID))
	}

	return nil
}

func (n *Neo4jPostRepository) DeletePost(ctx context.Context, postID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (p:Post {id: $id})
		DETACH DELETE p`, map[string]interface{}{
		"id": postID,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().NodesDeleted() < 0 {
		return types.NewNotFound(fmt.Sprintf("Post with id %s does not exist", postID))
	}

	return nil
}

type InteractionRepository interface {
	AddFriend(ctx context.Context, userID, friendID string) error
	RemoveFriend(ctx context.Context, userID, friendID string) error
	FollowUser(ctx context.Context, userID, followedUserID string) error
	UnfollowUser(ctx context.Context, userID, followedID string) error
	LikePost(ctx context.Context, userID, postID string) error
	UnlikePost(ctx context.Context, userID, postID string) error
	ViewPost(ctx context.Context, userID, postID string) error
}

type Neo4jInteractionRepository struct {
	driver neo4j.DriverWithContext
}

func NewNeo4jInteractionRepository(ctx context.Context, driver neo4j.DriverWithContext) (*Neo4jInteractionRepository, error) {
	storage := &Neo4jInteractionRepository{
		driver: driver,
	}

	session := driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	statements := []string{
		"CREATE INDEX post_likes_index IF NOT EXISTS FOR (p:Post) ON (p.likes);",
		"CREATE INDEX post_views_index IF NOT EXISTS FOR (p:Post) ON (p.views);",
	}

	for _, stmt := range statements {
		if _, err := session.Run(ctx, stmt, nil); err != nil {
			return nil, err
		}
	}

	return storage, nil
}

func (n *Neo4jInteractionRepository) AddFriend(ctx context.Context, userID, friendID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (u1:User {id: $userID}), (u2:User {id: $friendID})
		MERGE (u1)-[:FRIEND]->(u2)
		MERGE (u2)-[:FRIEND]->(u1)`, map[string]interface{}{
		"userID":   userID,
		"friendID": friendID,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().RelationshipsCreated() < 0 {
		return types.NewNotFound(fmt.Sprintf("Relationship could not be created for user %s and %s", userID, friendID))
	}

	return nil
}

func (n *Neo4jInteractionRepository) RemoveFriend(ctx context.Context, userID, friendID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (u1:User {id: $userID})-[r:FRIEND]-(u2:User {id: $friendID})
		DELETE r`, map[string]interface{}{
		"userID":   userID,
		"friendID": friendID,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().NodesDeleted() < 0 {
		return types.NewNotFound(fmt.Sprintf("Relationship could not be deleted for user %s and %s", userID, friendID))
	}

	return nil
}

func (n *Neo4jInteractionRepository) FollowUser(ctx context.Context, userID, followedID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (u1:User {id: $userID}), (u2:User {id: $followedID})
		MERGE (u1)-[:FOLLOWS]->(u2)`, map[string]interface{}{
		"userID":     userID,
		"followedID": followedID,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().RelationshipsCreated() < 0 {
		return types.NewNotFound(fmt.Sprintf("Relationship could not be created for user %s and %s", userID, followedID))
	}

	return nil
}

func (n *Neo4jInteractionRepository) UnfollowUser(ctx context.Context, userID, followedID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (u1:User {id: $userID})-[r:FOLLOWS]->(u2:User {id: $followedID})
		DELETE r`, map[string]interface{}{
		"userID":     userID,
		"followedID": followedID,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().RelationshipsDeleted() < 0 {
		return types.NewNotFound(fmt.Sprintf("Relationship could not be deleted for user %s and %s", userID, followedID))
	}

	return nil
}

func (n *Neo4jInteractionRepository) LikePost(ctx context.Context, userID, postID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (u:User {id: $userID}), (p:Post {id: $postID})
		MERGE (u)-[:LIKED]->(p)
		ON CREATE SET p.likes = p.likes + 1`, map[string]interface{}{
		"userID": userID,
		"postID": postID,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().RelationshipsCreated() < 0 {
		return types.NewNotFound(fmt.Sprintf("Relationship could not be created for user %s and %s", userID, postID))
	}

	return nil
}

func (n *Neo4jInteractionRepository) UnlikePost(ctx context.Context, userID, postID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (u:User {id: $userID})-[r:LIKED]->(p:Post {id: $postID})
		DELETE r
		WITH p, CASE WHEN r IS NOT NULL THEN 1 ELSE 0 END AS wasLiked
		SET p.likes = p.likes - wasLiked`, map[string]interface{}{
		"userID": userID,
		"postID": postID,
	})
	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().RelationshipsDeleted() < 0 {
		return types.NewNotFound(fmt.Sprintf("Relationship could not be deleted for user %s and %s", userID, postID))
	}

	return nil
}

func (n *Neo4jInteractionRepository) ViewPost(ctx context.Context, userID, postID string) error {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeWrite})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	result, err := session.Run(ctx, `
		MATCH (u:User {id: $userID}), (p:Post {id: $postID})
		MERGE (u)-[:VIEWED]->(p)
		ON CREATE SET p.views = p.views + 1`, map[string]interface{}{
		"userID": userID,
		"postID": postID,
	})

	if err != nil {
		return types.NewInternalError(err)
	}

	resultSummary, err := result.Consume(ctx)
	if err != nil {
		return types.NewInternalError(err)
	}

	if resultSummary.Counters().RelationshipsCreated() < 0 {
		return types.NewNotFound(fmt.Sprintf("Relationship could not be created for user %s and %s", userID, postID))
	}

	return nil
}

type RecommendationService interface {
	GenerateRecommendedPostIDs(ctx context.Context, userID string, minLikes uint32, limit uint32, next string) (*proto.RecommendedPosts, error)
}

type Neo4jRecommendationService struct {
	driver neo4j.DriverWithContext
}

func NewNeo4jRecommendationService(driver neo4j.DriverWithContext) *Neo4jRecommendationService {
	return &Neo4jRecommendationService{driver: driver}
}

func (n *Neo4jRecommendationService) GenerateRecommendedPostIDs(ctx context.Context, userID string, minLikes uint32, limit uint32, next string) (*proto.RecommendedPosts, error) {
	session := n.driver.NewSession(ctx, neo4j.SessionConfig{AccessMode: neo4j.AccessModeRead})
	defer func(session neo4j.SessionWithContext, ctx context.Context) {
		err := session.Close(ctx)
		if err != nil {
			log.Warn().Ctx(ctx).Msg("Failed to close session")
		}
	}(session, ctx)

	var postIDs []string
	var lastID string
	var hasNextPage bool

	_, err := session.ExecuteRead(ctx, func(tx neo4j.ManagedTransaction) (any, error) {
		requestLimit := limit + 1

		result, err := tx.Run(ctx, `
			CALL {
			   MATCH (user:User {id: $userID})-[:FRIEND]->(friend:User)-[:CREATED]->(post:Post)
			   WHERE ($cursor = "" OR post.id < $cursor) AND friend.id <> user.id
			   RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'friends' AS source
			   ORDER BY post.id DESC
			   LIMIT $limit
			
			   UNION
			
			   MATCH (user:User {id: $userID})-[:FOLLOWS]->(followed:User)-[:CREATED]->(post:Post)
			   WHERE ($cursor = "" OR post.id < $cursor) AND followed.id <> user.id
			   RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'followers' AS source
			   ORDER BY post.id DESC
			   LIMIT $limit
			
			   UNION
			
			   MATCH (user:User {id: $userID})-[:LIKED|VIEWED]->(likedPost:Post)<-[:CREATED]-(author:User)
			   WITH user, collect(distinct likedPost.tags) AS userTags
			   MATCH (post:Post)
			   WHERE any(tag IN post.tags WHERE tag IN userTags) AND ($cursor = "" OR post.id < $cursor)
			   RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'tags' AS source
			   ORDER BY post.id DESC
			   LIMIT $limit
			
			   UNION
			
			   MATCH (user:User)-[:CREATED]->(post:Post)
			   WHERE post.likes >= $minLikes AND ($cursor = "" OR post.id < $cursor) AND user.id <> $userID
			   RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'popular' AS source
			   ORDER BY post.likes DESC, post.id DESC
			   LIMIT $limit
			
			   UNION
			
			   MATCH (user:User)-[:CREATED]->(post:Post)
			   WHERE ($cursor = "" OR post.id < $cursor) AND user.id <> $userID
			   RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'recent' AS source
			   ORDER BY post.id DESC
			   LIMIT $limit
			}
			
			WITH postID, MAX(likes) AS likes, MAX(createdAt) AS createdAt, COLLECT(source)[0] AS source
			RETURN postID, likes, createdAt, source
			ORDER BY postID DESC
			LIMIT $limit
		`, map[string]any{
			"userID":   userID,
			"cursor":   next,
			"limit":    requestLimit,
			"minLikes": minLikes,
		})
		if err != nil {
			return nil, err
		}

		count := 0
		for result.Next(ctx) {
			count++
			record := result.Record()
			postIDStr, _ := record.Get("postID")

			if count <= int(limit) {
				postIDs = append(postIDs, postIDStr.(string))
			}

			lastID = postIDStr.(string)
		}

		hasNextPage = count > int(limit)
		if !hasNextPage {
			lastID = ""
		}
		return nil, result.Err()
	})

	if err != nil {
		return nil, err
	}

	return &proto.RecommendedPosts{
		PostIds:     postIDs,
		PerPage:     limit,
		HasNextPage: hasNextPage,
		Next:        lastID,
	}, nil
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
