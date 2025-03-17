package decorators

import (
	"context"
	"fmt"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/proto/github.com/mqsrr/zylo/feed-service/proto"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
	"sync/atomic"
	"time"
)

type ObservableFeedServiceServer struct {
	proto.UnimplementedFeedServiceServer
	inner          proto.FeedServiceServer
	requestCounter metric.Int64Counter
	requestLatency metric.Float64Histogram
	activeRequests int64
	attributes     attribute.Set
}

func NewObservableFeedServiceServer(inner proto.FeedServiceServer, meterProvider metric.MeterProvider) (*ObservableFeedServiceServer, error) {
	meter := meterProvider.Meter(config.DefaultConfig.ServiceName)
	requestCounter, err := meter.Int64Counter("grpc_server_requests_total", metric.WithDescription("Total number of gRPC requests"))
	if err != nil {
		return nil, err
	}

	requestLatency, err := meter.Float64Histogram("grpc_server_request_duration_seconds",
		metric.WithDescription("Request processing duration"),
		metric.WithExplicitBucketBoundaries(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0))

	if err != nil {
		return nil, err
	}

	containerId, err := config.GetContainerID()
	if err != nil {
		containerId = "0.0.0.0"
	}

	attributes := attribute.NewSet(attribute.String("service", config.DefaultConfig.ServiceName),
		attribute.String("instance", fmt.Sprintf("%s:%s", containerId, config.DefaultConfig.Port)),
		attribute.String("env", config.DefaultConfig.Environment))

	var activeRequests int64
	_, err = meter.Int64ObservableGauge("grpc_server_active_requests",
		metric.WithDescription("Active gRPC requests"),
		metric.WithInt64Callback(func(ctx context.Context, o metric.Int64Observer) error {
			o.Observe(activeRequests, metric.WithAttributeSet(attributes))
			return nil
		}))

	if err != nil {
		return nil, err
	}

	return &ObservableFeedServiceServer{
		inner:          inner,
		requestCounter: requestCounter,
		requestLatency: requestLatency,
		activeRequests: activeRequests,
		attributes:     attributes,
	}, nil
}

func (s *ObservableFeedServiceServer) GetPostsRecommendations(ctx context.Context, request *proto.GetRecommendedPostsRequest) (*proto.RecommendedPosts, error) {
	atomic.AddInt64(&s.activeRequests, 1)
	startTime := time.Now()

	ok, err := s.inner.GetPostsRecommendations(ctx, request)
	duration := time.Since(startTime).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	s.requestCounter.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("method", "GetPostsRecommendations"),
			attribute.String("status", status)),
		metric.WithAttributeSet(s.attributes))

	s.requestLatency.Record(ctx, duration, metric.WithAttributes(
		attribute.String("method", "GetPostsRecommendations"),
		attribute.String("status", status)),
		metric.WithAttributeSet(s.attributes))

	atomic.AddInt64(&s.activeRequests, -1)
	return ok, err
}
