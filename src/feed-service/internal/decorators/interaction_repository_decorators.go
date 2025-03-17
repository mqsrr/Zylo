package decorators

import (
	"context"
	"fmt"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/storage"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	semconv "go.opentelemetry.io/otel/semconv/v1.27.0"
	"go.opentelemetry.io/otel/trace"
	"sync/atomic"
	"time"
)

type ObservableInteractionRepository struct {
	inner               storage.InteractionRepository
	tracer              trace.Tracer
	requestCounter      metric.Int64Counter
	requestLatency      metric.Float64Histogram
	activeDbConnections int64
	attributes          attribute.Set
}

func NewObservableInteractionRepository(inner storage.InteractionRepository, traceProvider trace.TracerProvider, meterProvider metric.MeterProvider) (*ObservableInteractionRepository, error) {
	tracer := traceProvider.Tracer(config.DefaultConfig.ServiceName)
	meter := meterProvider.Meter(config.DefaultConfig.ServiceName)

	requestCounter, err := meter.Int64Counter("db_queries_total", metric.WithDescription("Total number of database queries"))
	if err != nil {
		return nil, err
	}

	requestLatency, err := meter.Float64Histogram("db_query_duration_seconds",
		metric.WithDescription("Query execution duration"),
		metric.WithExplicitBucketBoundaries(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0))

	if err != nil {
		return nil, err
	}

	containerId, err := config.GetContainerID()
	if err != nil {
		containerId = "0.0.0.0"
	}

	attributes := attribute.NewSet(
		attribute.String("service", config.DefaultConfig.ServiceName),
		attribute.String("instance", fmt.Sprintf("%s:%s", containerId, config.DefaultConfig.Port)),
		attribute.String("db", "neo4j"),
		attribute.String("env", config.DefaultConfig.Environment),
	)

	var activeRequests int64
	_, err = meter.Int64ObservableGauge("db_connections",
		metric.WithDescription("Active database connections"),
		metric.WithInt64Callback(func(ctx context.Context, o metric.Int64Observer) error {
			o.Observe(activeRequests, metric.WithAttributeSet(attributes))
			return nil
		}))

	return &ObservableInteractionRepository{
		inner:          inner,
		tracer:         tracer,
		requestCounter: requestCounter,
		requestLatency: requestLatency,
		attributes:     attributes,
	}, nil
}

func (o ObservableInteractionRepository) AddFriend(ctx context.Context, userID, friendID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.create friend",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("relationships"),
			semconv.DBOperationName("MERGE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("friend_id", friendID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.AddFriend(ctx, userID, friendID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "AddFriend"),
			attribute.String("query_type", "MERGE"),
			attribute.String("table", "relationships"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "AddFriend"),
		attribute.String("query_type", "MERGE"),
		attribute.String("table", "relationships"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservableInteractionRepository) RemoveFriend(ctx context.Context, userID, friendID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.remove friend",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("relationships"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("friend_id", friendID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.RemoveFriend(ctx, userID, friendID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "RemoveFriend"),
			attribute.String("query_type", "DELETE"),
			attribute.String("table", "relationships"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "RemoveFriend"),
		attribute.String("query_type", "DELETE"),
		attribute.String("table", "relationships"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservableInteractionRepository) FollowUser(ctx context.Context, userID, followedUserID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.create follower",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("relationships"),
			semconv.DBOperationName("MERGE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("followed_user_id", followedUserID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.FollowUser(ctx, userID, followedUserID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "FollowUser"),
			attribute.String("query_type", "MERGE"),
			attribute.String("table", "relationships"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "FollowUser"),
		attribute.String("query_type", "MERGE"),
		attribute.String("table", "relationships"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservableInteractionRepository) UnfollowUser(ctx context.Context, userID, followedID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.delete follower",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("relationships"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("followed_id", followedID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.UnfollowUser(ctx, userID, followedID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "UnfollowUser"),
			attribute.String("query_type", "DELETE"),
			attribute.String("table", "relationships"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "UnfollowUser"),
		attribute.String("query_type", "DELETE"),
		attribute.String("table", "relationships"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservableInteractionRepository) LikePost(ctx context.Context, userID, postID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.create like",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("relationships"),
			semconv.DBOperationName("MERGE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("post_id", postID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.LikePost(ctx, userID, postID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "LikePost"),
			attribute.String("query_type", "MERGE"),
			attribute.String("table", "relationships"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "LikePost"),
		attribute.String("query_type", "MERGE"),
		attribute.String("table", "relationships"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservableInteractionRepository) UnlikePost(ctx context.Context, userID, postID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.delete like",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("relationships"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("post_id", postID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.UnlikePost(ctx, userID, postID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, 1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "UnlikePost"),
			attribute.String("query_type", "DELETE"),
			attribute.String("table", "relationships"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "UnlikePost"),
		attribute.String("query_type", "DELETE"),
		attribute.String("table", "relationships"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservableInteractionRepository) ViewPost(ctx context.Context, userID, postID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.create view",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("relationships"),
			semconv.DBOperationName("MERGE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("post_id", postID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.ViewPost(ctx, userID, postID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "ViewPost"),
			attribute.String("query_type", "MERGE"),
			attribute.String("table", "relationships"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "ViewPost"),
		attribute.String("query_type", "MERGE"),
		attribute.String("table", "relationships"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}
