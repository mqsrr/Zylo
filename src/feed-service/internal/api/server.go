package api

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/decorators"
	"github.com/mqsrr/zylo/feed-service/internal/mq"
	"github.com/mqsrr/zylo/feed-service/internal/proto/github.com/mqsrr/zylo/feed-service/proto"
	"github.com/mqsrr/zylo/feed-service/internal/storage"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"github.com/rs/zerolog/log"
	log2 "go.opentelemetry.io/otel/sdk/log"
	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/trace"
	"google.golang.org/grpc"
	"net"
)

type Server struct {
	config                *config.Config
	grpcServer            *grpc.Server
	userRepository        storage.UserRepository
	postRepository        storage.PostRepository
	interactionRepository storage.InteractionRepository
	traceProvider         *trace.TracerProvider
	meterProvider         *metric.MeterProvider
	loggerProvider        *log2.LoggerProvider
	consumer              mq.Consumer
}

func NewServer(ctx context.Context,
	cfg *config.Config,
	consumer mq.Consumer,
	grpcServer *grpc.Server,
	traceProvider *trace.TracerProvider,
	meterProvider *metric.MeterProvider,
	loggerProvider *log2.LoggerProvider,
	driver neo4j.DriverWithContext) (*Server, error) {
	userRepository, err := storage.NewNeo4jUserRepository(ctx, driver)
	if err != nil {
		return nil, err
	}
	decoratedUserRepository, err := decorators.NewObservableUserRepository(userRepository, traceProvider, meterProvider)
	if err != nil {
		return nil, err
	}

	postRepository, err := storage.NewNeo4jPostRepository(ctx, driver)
	if err != nil {
		return nil, err
	}

	decoratedPostRepository, err := decorators.NewObservablePostRepository(postRepository, traceProvider, meterProvider)
	if err != nil {
		return nil, err
	}

	interactionRepository, err := storage.NewNeo4jInteractionRepository(ctx, driver)
	if err != nil {
		return nil, err
	}

	decoratedInteractionRepository, err := decorators.NewObservableInteractionRepository(interactionRepository, traceProvider, meterProvider)
	if err != nil {
		return nil, err
	}

	return &Server{
		config:                cfg,
		grpcServer:            grpcServer,
		userRepository:        decoratedUserRepository,
		postRepository:        decoratedPostRepository,
		interactionRepository: decoratedInteractionRepository,
		traceProvider:         traceProvider,
		meterProvider:         meterProvider,
		loggerProvider:        loggerProvider,
		consumer:              consumer,
	}, nil
}

func (s *Server) MountHandlers(feedServer *FeedServiceServer) error {
	if err := s.HandleUserMessages(); err != nil {
		return err
	}

	if err := s.HandlePostMessages(); err != nil {
		return err
	}

	if err := s.HandlePostInteractionMessages(); err != nil {
		return err
	}

	if err := s.HandleUserSocialMessages(); err != nil {
		return err
	}

	observableGrpcServer, err := decorators.NewObservableFeedServiceServer(feedServer, s.meterProvider)
	if err != nil {
		return err
	}

	proto.RegisterFeedServiceServer(s.grpcServer, observableGrpcServer)
	return nil
}

func (s *Server) ListenAndServe() error {
	go func() {
		lis, err := net.Listen("tcp", s.config.GrpcServer.Port)
		if err != nil {
			panic(err)
		}

		log.Info().Msgf("gRPC server is listening on %s", s.config.GrpcServer.Port)
		if err := s.grpcServer.Serve(lis); err != nil {
			panic(err)
		}
	}()

	return nil
}

func (s *Server) Shutdown(ctx context.Context) error {
	if err := s.consumer.Shutdown(); err != nil {
		return err
	}

	if err := s.traceProvider.Shutdown(ctx); err != nil {
		return err
	}

	if err := s.meterProvider.Shutdown(ctx); err != nil {
		return err
	}

	if err := s.loggerProvider.Shutdown(ctx); err != nil {
		return err
	}

	s.grpcServer.GracefulStop()
	return nil
}
