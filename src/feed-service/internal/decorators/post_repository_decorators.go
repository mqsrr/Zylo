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

type ObservablePostRepository struct {
	inner               storage.PostRepository
	tracer              trace.Tracer
	requestCounter      metric.Int64Counter
	requestLatency      metric.Float64Histogram
	activeDbConnections int64
	attributes          attribute.Set
}

func NewObservablePostRepository(inner storage.PostRepository, traceProvider trace.TracerProvider, meterProvider metric.MeterProvider) (*ObservablePostRepository, error) {
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

	return &ObservablePostRepository{
		inner:          inner,
		tracer:         tracer,
		requestCounter: requestCounter,
		requestLatency: requestLatency,
		attributes:     attributes,
	}, nil
}

func (o ObservablePostRepository) CreatePost(ctx context.Context, postID string, userID string, content string, createdAt time.Time) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.create post",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("posts"),
			semconv.DBOperationName("MERGE"),
			semconv.DBSystemNeo4j,
			attribute.String("user_id", userID),
			attribute.String("post_id", postID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.CreatePost(ctx, postID, userID, content, createdAt)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "CreatePost"),
			attribute.String("query_type", "MERGE"),
			attribute.String("table", "posts"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "CreatePost"),
		attribute.String("query_type", "MERGE"),
		attribute.String("table", "posts"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservablePostRepository) UpdatePostTags(ctx context.Context, postID string, content string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.update tags",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("posts"),
			semconv.DBOperationName("SET"),
			semconv.DBSystemNeo4j,
			attribute.String("post_id", postID)))
	defer span.End()

	startTime := time.Now()
	err := o.inner.UpdatePostTags(ctx, postID, content)
	duration := time.Since(startTime).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "UpdatePostTags"),
			attribute.String("query_type", "SET"),
			attribute.String("table", "posts"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "UpdatePostTags"),
		attribute.String("query_type", "SET"),
		attribute.String("table", "posts"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o ObservablePostRepository) DeletePost(ctx context.Context, postID string) error {
	ctx, span := o.tracer.Start(ctx, "neo4j.delete post",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("posts"),
			semconv.DBOperationName("DELETE"),
			semconv.DBSystemNeo4j,
			attribute.String("post_id", postID)))
	defer span.End()

	atomic.AddInt64(&o.activeDbConnections, 1)
	startTime := time.Now()

	err := o.inner.DeletePost(ctx, postID)

	duration := time.Since(startTime).Seconds()
	atomic.AddInt64(&o.activeDbConnections, -1)

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "DeletePost"),
			attribute.String("query_type", "DELETE"),
			attribute.String("table", "posts"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "DeletePost"),
		attribute.String("query_type", "DELETE"),
		attribute.String("table", "posts"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}
