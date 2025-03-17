package api

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/decorators"
	"github.com/mqsrr/zylo/feed-service/internal/proto/github.com/mqsrr/zylo/feed-service/proto"
	"github.com/mqsrr/zylo/feed-service/internal/storage"
	"github.com/mqsrr/zylo/feed-service/internal/types"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/trace"
)

type FeedServiceServer struct {
	proto.UnimplementedFeedServiceServer
	recommendationService storage.RecommendationService
}

func NewFeedServiceServer(driver neo4j.DriverWithContext, traceProvider trace.TracerProvider, meterProvider metric.MeterProvider) (*FeedServiceServer, error) {
	recommendationService := storage.NewNeo4jRecommendationService(driver)
	decoratedRecommendationService, err := decorators.NewRecommendationService(recommendationService, traceProvider, meterProvider)
	if err != nil {
		return nil, err
	}

	return &FeedServiceServer{
		recommendationService: decoratedRecommendationService,
	}, nil
}

func (s *FeedServiceServer) GetPostsRecommendations(ctx context.Context, r *proto.GetRecommendedPostsRequest) (*proto.RecommendedPosts, error) {
	perPage := r.GetPerPage()
	if perPage <= 0 {
		perPage = 10
	}

	minLikes := r.GetMinLikes()
	recommendedPosts, err := s.recommendationService.GenerateRecommendedPostIDs(ctx, r.UserId, minLikes, perPage, r.GetLastPostId())
	if err != nil {
		return nil, types.GrpcError(err)
	}
	return recommendedPosts, nil
}
