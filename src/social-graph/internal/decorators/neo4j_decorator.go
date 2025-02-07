package decorators

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/rs/zerolog/log"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	semconv "go.opentelemetry.io/otel/semconv/v1.26.0"
	"go.opentelemetry.io/otel/trace"
	"time"
)

type CachedNeo4jStorage struct {
	inner storage.RelationshipStorage
	cache storage.CacheStorage
	cfg   *config.RedisConfig
}

func NewCachedNeo4jStorage(inner storage.RelationshipStorage, cfg *config.RedisConfig, cache storage.CacheStorage) *CachedNeo4jStorage {
	return &CachedNeo4jStorage{
		inner: inner,
		cache: cache,
		cfg:   cfg,
	}
}

func (c *CachedNeo4jStorage) CreateUser(ctx context.Context, userID ulid.ULID) (bool, error) {
	return c.inner.CreateUser(ctx, userID)
}

func (c *CachedNeo4jStorage) DeleteUserByID(ctx context.Context, userID ulid.ULID) (bool, error) {
	if ok, err := c.inner.DeleteUserByID(ctx, userID); err != nil || !ok {
		return ok, err
	}

	if err := c.cache.Del(ctx, userID.String()); err != nil {
		log.Warn().Str("user_id", userID.String()).Err(err).Msg("failed to invalidate user cache")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) GetUserWithRelationships(ctx context.Context, userID ulid.ULID) (*types.UserWithRelationships, error) {
	var rel *types.UserWithRelationships
	userIDString := userID.String()
	cacheField := "relationships"

	if err := c.cache.HGet(ctx, userIDString, cacheField, &rel); err == nil {
		return rel, nil
	}

	rel, err := c.inner.GetUserWithRelationships(ctx, userID)
	if err != nil {
		return nil, err
	}

	if err := c.cache.HSet(ctx, userIDString, cacheField, rel, c.cfg.Expire); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to cache user relationships")
	}

	return rel, nil
}

func (c *CachedNeo4jStorage) BatchGetUserWithRelationships(ctx context.Context, userIDs []ulid.ULID) (map[ulid.ULID]*types.UserWithRelationships, error) {
	result := make(map[ulid.ULID]*types.UserWithRelationships, len(userIDs))

	var missingIDs []ulid.ULID
	cacheField := "relationships"

	for _, userID := range userIDs {
		var rel *types.UserWithRelationships
		userIDStr := userID.String()

		if err := c.cache.HGet(ctx, userIDStr, cacheField, &rel); err == nil && rel != nil {
			result[userID] = rel
			continue
		}

		missingIDs = append(missingIDs, userID)
	}

	if len(missingIDs) > 0 {
		dbResults, err := c.inner.BatchGetUserWithRelationships(ctx, missingIDs)
		if err != nil {
			return nil, err
		}

		for userID, rel := range dbResults {
			result[userID] = rel
			userIDStr := userID.String()
			if err := c.cache.HSet(ctx, userIDStr, cacheField, rel, c.cfg.Expire); err != nil {
				log.Warn().
					Str("user_id", userIDStr).
					Err(err).
					Msg("failed to cache user relationships")
			}
		}
	}

	return result, nil
}

func (c *CachedNeo4jStorage) GetFollowers(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	var rel *types.Relationship
	userIDString := userID.String()
	cacheField := "followers"

	if err := c.cache.HGet(ctx, userIDString, cacheField, &rel); err == nil {
		return rel, nil
	}

	rel, err := c.inner.GetFollowers(ctx, userID)
	if err != nil {
		return nil, err
	}

	if err := c.cache.HSet(ctx, userIDString, cacheField, rel, c.cfg.Expire); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to cache user followers")
	}

	return rel, nil
}

func (c *CachedNeo4jStorage) GetFollowedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	var rel *types.Relationship
	userIDString := userID.String()
	cacheField := "followed"

	if err := c.cache.HGet(ctx, userIDString, cacheField, &rel); err == nil {
		return rel, nil
	}

	rel, err := c.inner.GetFollowedPeople(ctx, userID)
	if err != nil {
		return nil, err
	}

	if err := c.cache.HSet(ctx, userIDString, cacheField, rel, c.cfg.Expire); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to cache user followed")
	}

	return rel, nil
}

func (c *CachedNeo4jStorage) GetBlockedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	var rel *types.Relationship
	userIDString := userID.String()
	cacheField := "blocked"

	if err := c.cache.HGet(ctx, userIDString, cacheField, &rel); err == nil {
		return rel, nil
	}

	rel, err := c.inner.GetBlockedPeople(ctx, userID)
	if err != nil {
		return nil, err
	}

	if err := c.cache.HSet(ctx, userIDString, cacheField, rel, c.cfg.Expire); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to cache user blocked")
	}

	return rel, nil
}

func (c *CachedNeo4jStorage) GetFriends(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	var rel *types.Relationship
	userIDString := userID.String()
	cacheField := "friends"

	if err := c.cache.HGet(ctx, userIDString, cacheField, &rel); err == nil {
		return rel, nil
	}

	rel, err := c.inner.GetFriends(ctx, userID)
	if err != nil {
		return nil, err
	}

	if err := c.cache.HSet(ctx, userIDString, cacheField, rel, c.cfg.Expire); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to cache user friends")
	}

	return rel, nil
}

func (c *CachedNeo4jStorage) GetPendingFriendRequests(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	var rel *types.Relationship
	userIDString := userID.String()
	cacheField := "pending-friend-requests"

	if err := c.cache.HGet(ctx, userIDString, cacheField, &rel); err == nil {
		return rel, nil
	}

	rel, err := c.inner.GetPendingFriendRequests(ctx, userID)
	if err != nil {
		return nil, err
	}

	if err := c.cache.HSet(ctx, userIDString, cacheField, rel, c.cfg.Expire); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to cache user pending friend requests")
	}

	return rel, nil
}

func (c *CachedNeo4jStorage) RemoveFriend(ctx context.Context, userID, friendID ulid.ULID) (bool, error) {
	userIDString := userID.String()
	friendIDString := friendID.String()
	cacheField := "friends"

	if ok, err := c.inner.RemoveFriend(ctx, userID, friendID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, userIDString, cacheField); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to invalidate cache for user friends")
	}

	if err := c.cache.HDelete(ctx, friendIDString, cacheField); err != nil {
		log.Warn().Str("user_id", friendIDString).Err(err).Msg("failed to invalidate cache for user friends")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) FollowUser(ctx context.Context, followerID, followedID ulid.ULID) (bool, error) {
	followerIDString := followerID.String()
	followedIDString := followedID.String()

	if ok, err := c.inner.FollowUser(ctx, followerID, followedID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, followerIDString, "followed"); err != nil {
		log.Warn().Str("user_id", followerIDString).Err(err).Msg("failed to invalidate cache for user following list")
	}

	if err := c.cache.HDelete(ctx, followedIDString, "followers"); err != nil {
		log.Warn().Str("user_id", followedIDString).Err(err).Msg("failed to invalidate cache for user followed list")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) UnfollowUser(ctx context.Context, followerID, followedID ulid.ULID) (bool, error) {
	followerIDString := followerID.String()
	followedIDString := followedID.String()

	if ok, err := c.inner.UnfollowUser(ctx, followerID, followedID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, followerIDString, "followed"); err != nil {
		log.Warn().Str("user_id", followerIDString).Err(err).Msg("failed to invalidate cache for user following list")
	}

	if err := c.cache.HDelete(ctx, followedIDString, "followers"); err != nil {
		log.Warn().Str("user_id", followedIDString).Err(err).Msg("failed to invalidate cache for user followed list")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) SendFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	userIDString := userID.String()
	receiverIDString := receiverID.String()

	if ok, err := c.inner.SendFriendRequest(ctx, userID, receiverID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, userIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to invalidate cache for user friend requests")
	}

	if err := c.cache.HDelete(ctx, receiverIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", receiverIDString).Err(err).Msg("failed to invalidate cache for user friend requests")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) AcceptFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	userIDString := userID.String()
	receiverIDString := receiverID.String()

	if ok, err := c.inner.AcceptFriendRequest(ctx, userID, receiverID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, userIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to invalidate cache for user friend requests")
	}

	if err := c.cache.HDelete(ctx, receiverIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", receiverIDString).Err(err).Msg("failed to invalidate cache for user friend requests")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) DeclineFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	userIDString := userID.String()
	receiverIDString := receiverID.String()

	if ok, err := c.inner.DeclineFriendRequest(ctx, userID, receiverID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, userIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", userIDString).Err(err).Msg("failed to invalidate cache for user friend requests")
	}

	if err := c.cache.HDelete(ctx, receiverIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", receiverIDString).Err(err).Msg("failed to invalidate cache for user friend requests")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) BlockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error) {
	blockerIDString := blockerID.String()
	blockedIDString := blockedID.String()

	if ok, err := c.inner.BlockUser(ctx, blockerID, blockedID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, blockerIDString, "blocks"); err != nil {
		log.Warn().Str("user_id", blockerIDString).Err(err).Msg("failed to invalidate cache for user blocks")
	}

	if err := c.cache.HDelete(ctx, blockedIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", blockedIDString).Err(err).Msg("failed to invalidate cache for user blocks")
	}

	return true, nil
}

func (c *CachedNeo4jStorage) UnblockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error) {
	blockerIDString := blockerID.String()
	blockedIDString := blockedID.String()

	if ok, err := c.inner.UnblockUser(ctx, blockerID, blockedID); !ok || err != nil {
		return false, err
	}

	if err := c.cache.HDelete(ctx, blockerIDString, "blocks"); err != nil {
		log.Warn().Str("user_id", blockerIDString).Err(err).Msg("failed to invalidate cache for user blocks")
	}

	if err := c.cache.HDelete(ctx, blockedIDString, "pending-friend-requests"); err != nil {
		log.Warn().Str("user_id", blockedIDString).Err(err).Msg("failed to invalidate cache for user blocks")
	}

	return true, nil
}

type ObservableNeo4jStorage struct {
	inner          storage.RelationshipStorage
	tracer         trace.Tracer
	requestCounter metric.Int64Counter
	requestLatency metric.Float64Histogram
}

func NewObservableNeo4jStorage(inner storage.RelationshipStorage, traceProvider trace.TracerProvider, meterProvider metric.MeterProvider) (*ObservableNeo4jStorage, error) {
	tracer := traceProvider.Tracer("social-graph")
	meter := meterProvider.Meter("social-graph")

	requestCounter, err := meter.Int64Counter("relationships_storage_request_total", metric.WithDescription("Total requests to RelationshipStorage"))
	if err != nil {
		return nil, err
	}

	requestLatency, err := meter.Float64Histogram("relationships_storage_duration_seconds",
		metric.WithDescription("Latency of RelationshipStorage"),
		metric.WithExplicitBucketBoundaries(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0))

	if err != nil {
		return nil, err
	}

	return &ObservableNeo4jStorage{
		inner:          inner,
		tracer:         tracer,
		requestCounter: requestCounter,
		requestLatency: requestLatency,
	}, nil
}

func (o *ObservableNeo4jStorage) CreateUser(ctx context.Context, userID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users create",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("MERGE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String())))
	defer span.End()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(
		attribute.String("method", "CreateUser"),
		attribute.String("operation", "MERGE")))

	startTime := time.Now()
	ok, err := o.inner.CreateUser(ctx, userID)

	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "DeleteUserByID")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, nil
}

func (o *ObservableNeo4jStorage) DeleteUserByID(ctx context.Context, userID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users delete",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "DeleteUserByID")))

	ok, err := o.inner.DeleteUserByID(ctx, userID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "DeleteUserByID")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}

func (o *ObservableNeo4jStorage) GetUserWithRelationships(ctx context.Context, userID ulid.ULID) (*types.UserWithRelationships, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users getRelationships",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("MATCH"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetUserWithRelationships")))
	res, err := o.inner.GetUserWithRelationships(ctx, userID)

	duration := time.Since(startTime).Seconds()
	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetUserWithRelationships")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return res, err
}

func (o *ObservableNeo4jStorage) BatchGetUserWithRelationships(ctx context.Context, userIDs []ulid.ULID) (map[ulid.ULID]*types.UserWithRelationships, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users batchGetRelationships",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("BATCH_SELECT"),
			semconv.DBSystemNeo4j,
			attribute.Int("batch_size", len(userIDs)),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "BatchGetUserWithRelationships")))
	res, err := o.inner.BatchGetUserWithRelationships(ctx, userIDs)

	duration := time.Since(startTime).Seconds()
	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "BatchGetUserWithRelationships")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return res, err
}

func (o *ObservableNeo4jStorage) GetFollowers(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users getFollowers",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("SELECT"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetFollowers")))
	res, err := o.inner.GetFollowers(ctx, userID)

	duration := time.Since(startTime).Seconds()
	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetFollowers")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return res, err
}

func (o *ObservableNeo4jStorage) GetFollowedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users getFollowed",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("SELECT"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetFollowedPeople")))
	res, err := o.inner.GetFollowedPeople(ctx, userID)

	duration := time.Since(startTime).Seconds()
	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetFollowedPeople")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return res, err
}

func (o *ObservableNeo4jStorage) GetBlockedPeople(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users getBlocked",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("SELECT"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetBlockedPeople")))

	res, err := o.inner.GetBlockedPeople(ctx, userID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetBlockedPeople")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return res, err
}

func (o *ObservableNeo4jStorage) GetFriends(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users getFriends",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("SELECT"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetFriends")))
	res, err := o.inner.GetFriends(ctx, userID)

	duration := time.Since(startTime).Seconds()
	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetFriends")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return res, err
}

func (o *ObservableNeo4jStorage) GetPendingFriendRequests(ctx context.Context, userID ulid.ULID) (*types.Relationship, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users getPendingFriendRequests",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("SELECT"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetPendingFriendRequests")))
	res, err := o.inner.GetPendingFriendRequests(ctx, userID)

	duration := time.Since(startTime).Seconds()
	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetPendingFriendRequests")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return res, err
}

func (o *ObservableNeo4jStorage) RemoveFriend(ctx context.Context, userID, friendID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users deleteFriend",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
			attribute.String("friend_id", friendID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "RemoveFriend")))

	ok, err := o.inner.RemoveFriend(ctx, userID, friendID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "RemoveFriend")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}

func (o *ObservableNeo4jStorage) FollowUser(ctx context.Context, followerID, followedID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users follow",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("INSERT"),
			semconv.DBSystemNeo4j,
			attribute.String("follower_id", followerID.String()),
			attribute.String("followed_id", followedID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "FollowUser")))

	ok, err := o.inner.FollowUser(ctx, followerID, followedID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "FollowUser")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}

func (o *ObservableNeo4jStorage) UnfollowUser(ctx context.Context, followerID, followedID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users unfollow",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("follower_id", followerID.String()),
			attribute.String("followed_id", followedID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "UnfollowUser")))

	ok, err := o.inner.UnfollowUser(ctx, followerID, followedID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "UnfollowUser")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}

func (o *ObservableNeo4jStorage) SendFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users sendFriendRequest",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("INSERT"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
			attribute.String("receiver_id", receiverID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "SendFriendRequest")))
	ok, err := o.inner.SendFriendRequest(ctx, userID, receiverID)

	duration := time.Since(startTime).Seconds()
	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "SendFriendRequest")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}

func (o *ObservableNeo4jStorage) AcceptFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users acceptFriendRequest",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("UPDATE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
			attribute.String("receiver_id", receiverID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "AcceptFriendRequest")))

	ok, err := o.inner.AcceptFriendRequest(ctx, userID, receiverID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "AcceptFriendRequest")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}
func (o *ObservableNeo4jStorage) DeclineFriendRequest(ctx context.Context, userID, receiverID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users declineFriendRequest",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID.String()),
			attribute.String("receiver_id", receiverID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "DeclineFriendRequest")))

	ok, err := o.inner.DeclineFriendRequest(ctx, userID, receiverID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "DeclineFriendRequest")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}

func (o *ObservableNeo4jStorage) BlockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users block",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("INSERT"),
			semconv.DBSystemNeo4j,
			attribute.String("blocker_id", blockerID.String()),
			attribute.String("blocked_id", blockedID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "BlockUser")))

	ok, err := o.inner.BlockUser(ctx, blockerID, blockedID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "BlockUser")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}

func (o *ObservableNeo4jStorage) UnblockUser(ctx context.Context, blockerID, blockedID ulid.ULID) (bool, error) {
	ctx, span := o.tracer.Start(ctx, "neo4j.users unblock",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("blocker_id", blockerID.String()),
			attribute.String("blocked_id", blockedID.String()),
		))
	defer span.End()

	startTime := time.Now()
	o.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "UnblockUser")))

	ok, err := o.inner.UnblockUser(ctx, blockerID, blockedID)
	duration := time.Since(startTime).Seconds()

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "UnblockUser")))
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, err
}
