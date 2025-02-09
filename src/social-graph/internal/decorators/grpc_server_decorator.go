package decorators

import (
	"context"
	"fmt"
	"github.com/mqsrr/zylo/social-graph/internal/protos/github.com/mqsrr/zylo/social-graph/proto"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	semconv "go.opentelemetry.io/otel/semconv/v1.27.0"
	"go.opentelemetry.io/otel/trace"
	"time"
)

type ObservableRelationshipServiceServer struct {
	proto.UnimplementedRelationshipServiceServer
	inner          proto.RelationshipServiceServer
	tracer         trace.Tracer
	requestCounter metric.Int64Counter
	requestLatency metric.Float64Histogram
}

func NewObservableRelationshipServer(inner proto.RelationshipServiceServer, traceProvider trace.TracerProvider, meterProvider metric.MeterProvider) (*ObservableRelationshipServiceServer, error) {
	tracer := traceProvider.Tracer("social-graph-grpc")
	meter := meterProvider.Meter("social-graph-grpc")
	requestCounter, err := meter.Int64Counter("relationship_grpc_server_request_total", metric.WithDescription("Total requests to RelationshipService"))
	if err != nil {
		return nil, err
	}

	requestLatency, err := meter.Float64Histogram("relationship_grpc_server_duration_seconds",
		metric.WithDescription("Latency of RelationshipService"),
		metric.WithExplicitBucketBoundaries(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0))

	if err != nil {
		return nil, err
	}

	return &ObservableRelationshipServiceServer{
		inner:          inner,
		tracer:         tracer,
		requestCounter: requestCounter,
		requestLatency: requestLatency,
	}, nil
}

func (s *ObservableRelationshipServiceServer) GetUserRelationships(ctx context.Context, req *proto.RelationshipRequest) (*proto.RelationshipResponse, error) {
	ctx, span := s.tracer.Start(ctx, fmt.Sprintf("RelationshipService/GetUserRelationships"),
		trace.WithSpanKind(trace.SpanKindServer),
		trace.WithAttributes(
			semconv.RPCSystemGRPC,
			semconv.ServerAddress("127.0.0.0"),
			semconv.ServerPort(50051),
			semconv.NetworkTransportTCP,
			semconv.RPCMethod("GetUserRelationships"),
			attribute.String("user_id", req.GetUserId())))
	defer span.End()
	s.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetUserRelationships")))

	startTime := time.Now()
	ok, err := s.inner.GetUserRelationships(ctx, req)

	duration := time.Since(startTime).Seconds()

	s.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetUserRelationships")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, nil
}

func (s *ObservableRelationshipServiceServer) GetBatchRelationships(ctx context.Context, req *proto.BatchRelationshipRequest) (*proto.BatchRelationshipResponse, error) {
	ctx, span := s.tracer.Start(ctx, fmt.Sprintf("RelationshipService/GetBatchRelationships"),
		trace.WithSpanKind(trace.SpanKindServer),
		trace.WithAttributes(
			semconv.RPCSystemGRPC,
			semconv.ServerAddress("127.0.0.0"),
			semconv.ServerPort(50051),
			semconv.NetworkTransportTCP,
			semconv.RPCMethod("GetBatchRelationships"),
			attribute.StringSlice("user_id", req.GetUserIds())))
	defer span.End()
	s.requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", "GetBatchRelationships")))

	startTime := time.Now()
	ok, err := s.inner.GetBatchRelationships(ctx, req)

	duration := time.Since(startTime).Seconds()

	s.requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", "GetBatchRelationships")))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetStatus(codes.Ok, "")
	return ok, nil
}
