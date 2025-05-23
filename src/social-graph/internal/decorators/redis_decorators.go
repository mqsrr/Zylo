package decorators

import (
	"context"
	"fmt"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	semconv "go.opentelemetry.io/otel/semconv/v1.26.0"
	"go.opentelemetry.io/otel/trace"
	"time"
)

type ObservableRedisStorage struct {
	inner          storage.CacheStorage
	tracer         trace.Tracer
	requestCounter metric.Int64Counter
	requestLatency metric.Float64Histogram
	attributes     attribute.Set
}

func NewObservableRedisStorage(inner storage.CacheStorage, traceProvider trace.TracerProvider, meterProvider metric.MeterProvider) (*ObservableRedisStorage, error) {
	tracer := traceProvider.Tracer(config.DefaultConfig.ServiceName)
	meter := meterProvider.Meter(config.DefaultConfig.ServiceName)

	requestCounter, err := meter.Int64Counter("cache_operations_total", metric.WithDescription("Total cache operations (set/get/delete)"))
	if err != nil {
		return nil, err
	}

	requestLatency, err := meter.Float64Histogram("cache_operation_duration_seconds",
		metric.WithDescription("Time taken for cache operations"),
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
		attribute.String("cache", "redis"),
		attribute.String("env", config.DefaultConfig.Environment),
	)

	return &ObservableRedisStorage{
		inner:          inner,
		tracer:         tracer,
		requestCounter: requestCounter,
		requestLatency: requestLatency,
		attributes:     attributes,
	}, nil
}

func (o *ObservableRedisStorage) HSet(ctx context.Context, key, field string, v any, expire time.Duration) error {
	ctx, span := o.tracer.Start(ctx, "redis.users hset",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("HSET HEXPIRE"),
			semconv.DBSystemRedis,
			attribute.String("key", key), attribute.String("field", field)))
	defer span.End()

	startTime := time.Now()
	err := o.inner.HSet(ctx, key, field, v, expire)
	duration := time.Since(startTime).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "HSet"),
			attribute.String("operation", "HSET HEXPIRE"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "HSet"),
		attribute.String("operation", "HSET HEXPIRE"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o *ObservableRedisStorage) HGet(ctx context.Context, key, field string, v any) error {
	ctx, span := o.tracer.Start(ctx, "redis.users hget",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("HGET"),
			semconv.DBSystemRedis,
			attribute.String("key", key),
			attribute.String("field", field)))
	defer span.End()

	startTime := time.Now()
	err := o.inner.HGet(ctx, key, field, v)
	duration := time.Since(startTime).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "HGet"),
			attribute.String("operation", "HGET"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "HGet"),
		attribute.String("operation", "HGET"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o *ObservableRedisStorage) HDelete(ctx context.Context, key string, fields ...string) error {
	ctx, span := o.tracer.Start(ctx, "redis.users hdel",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("HDEL"),
			semconv.DBSystemRedis,
			attribute.String("key", key), attribute.String("field", "multiple")))
	defer span.End()

	startTime := time.Now()
	err := o.inner.HDelete(ctx, key, fields...)
	duration := time.Since(startTime).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "HDelete"),
			attribute.String("operation", "HDEL"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "HDelete"),
		attribute.String("operation", "HDEL"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o *ObservableRedisStorage) HDeleteAll(ctx context.Context, key, pattern string) error {
	ctx, span := o.tracer.Start(ctx, "redis.users hscan hdel",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("users"),
			semconv.DBOperationName("HSCAN HDEL"),
			semconv.DBSystemRedis,
			attribute.String("key", key), attribute.String("pattern", pattern)))
	defer span.End()

	startTime := time.Now()
	err := o.inner.HDeleteAll(ctx, key, pattern)
	duration := time.Since(startTime).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "HDeleteAll"),
			attribute.String("operation", "HSCAN HDEL"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "HDeleteAll"),
		attribute.String("operation", "HSCAN HDEL"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}

func (o *ObservableRedisStorage) Del(ctx context.Context, keys ...string) error {
	ctx, span := o.tracer.Start(ctx, "redis del",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			semconv.DBCollectionName("global"),
			semconv.DBOperationName("DEL"),
			semconv.DBSystemRedis,
			attribute.String("key", "multiple")))
	defer span.End()

	startTime := time.Now()
	err := o.inner.Del(ctx, keys...)
	duration := time.Since(startTime).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	o.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "Del"),
			attribute.String("operation", "DEL"),
			attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	o.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "Del"),
		attribute.String("operation", "DEL"),
		attribute.String("status", status)),
		metric.WithAttributeSet(o.attributes))

	if err != nil {
		span.RecordError(err)
		return err
	}

	span.SetStatus(codes.Ok, "")
	return nil
}
