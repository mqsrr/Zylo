package api

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/jwtauth/v5"
	"github.com/lestrrat-go/jwx/v2/jwt"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/decorators"
	m "github.com/mqsrr/zylo/social-graph/internal/middleware"
	"github.com/mqsrr/zylo/social-graph/internal/mq"
	"github.com/mqsrr/zylo/social-graph/internal/protos/github.com/mqsrr/zylo/social-graph/proto"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"github.com/rs/zerolog/log"
	log2 "go.opentelemetry.io/otel/sdk/log"
	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/trace"
	"google.golang.org/grpc"
	"net"
	"net/http"
	"time"
)

type Server struct {
	*chi.Mux
	cfg            *config.Config
	storage        storage.RelationshipStorage
	grpcServer     *grpc.Server
	cache          storage.CacheStorage
	consumer       mq.Consumer
	traceProvider  *trace.TracerProvider
	meterProvider  *metric.MeterProvider
	loggerProvider *log2.LoggerProvider
	httpServer     *http.Server
}

func ResponseWithJSON(w http.ResponseWriter, statusCode int, content any) {
	w.Header().Add("Content-Type", "application/json")
	w.WriteHeader(statusCode)

	if content != nil {
		jsonContent, _ := json.Marshal(content)
		w.Write(jsonContent)
	}
}

func NewServer(config *config.Config, storage storage.RelationshipStorage, grpcServer *grpc.Server, cache storage.CacheStorage, consumer mq.Consumer, traceProvider *trace.TracerProvider, meterProvider *metric.MeterProvider, loggerProvider *log2.LoggerProvider) *Server {
	srv := &Server{
		cfg:            config,
		storage:        storage,
		grpcServer:     grpcServer,
		cache:          cache,
		consumer:       consumer,
		traceProvider:  traceProvider,
		meterProvider:  meterProvider,
		loggerProvider: loggerProvider,
	}

	srv.httpServer = &http.Server{
		Addr:    config.Port,
		Handler: srv,
	}

	return srv
}

func setupJWT(config *config.Jwt) *jwtauth.JWTAuth {
	return jwtauth.New(
		"HS256",
		[]byte(config.Secret),
		nil,
		jwt.WithAcceptableSkew(5*time.Second),
		jwt.WithAudience(config.Audience),
		jwt.WithIssuer(config.Issuer))
}

func (s *Server) MountHandlers() error {
	tokenAuth := setupJWT(s.cfg.Jwt)
	r := chi.NewRouter()

	r.Use(
		jwtauth.Verifier(tokenAuth),
		jwtauth.Authenticator(tokenAuth),
		middleware.RequestID,
		middleware.RealIP,
		middleware.Recoverer,
		m.RequestLogger,
		m.OtelMiddleware(s.meterProvider),
	)

	r.Route("/api/users/{id}", func(r chi.Router) {
		r.Get("/relationships", m.MustUlidParams(s.HandleGetUserWithRelationships))

		r.Route("/followers", func(r chi.Router) {
			r.Get("/", m.MustUlidParams(s.HandleGetFollowers))
			r.Get("/me", m.MustUlidParams(s.HandleGetFollowedPeople))
			r.Post("/{followedId}", m.MustUlidParams(s.HandleFollowUser, "followedId"))
			r.Delete("/{followedId}", m.MustUlidParams(s.HandleUnfollowUser, "followedId"))
		})
		r.Route("/friends", func(r chi.Router) {
			r.Get("/", m.MustUlidParams(s.HandleGetFriends))
			r.Delete("/{friendId}", m.MustUlidParams(s.HandleRemoveFriend, "friendId"))
			r.Get("/requests", m.MustUlidParams(s.HandleGetPendingFriendRequests))
			r.Post("/requests/{receiverId}", m.MustUlidParams(s.HandleSendFriendRequest, "receiverId"))
			r.Put("/requests/{receiverId}", m.MustUlidParams(s.HandleAcceptFriendRequest, "receiverId"))
			r.Delete("/requests/{receiverId}", m.MustUlidParams(s.HandleDeclineFriendRequest, "receiverId"))
		})
		r.Route("/blocks", func(r chi.Router) {
			r.Get("/", m.MustUlidParams(s.HandleGetBlockedPeople))
			r.Post("/{blockedId}", m.MustUlidParams(s.HandleBlockUser, "blockedId"))
			r.Delete("/{blockedId}", m.MustUlidParams(s.HandleUnblockUser, "blockedId"))
		})
	})
	if err := s.HandleUserCreatedMessage(); err != nil {
		return err
	}

	if err := s.HandleUserDeletedMessage(); err != nil {
		return err
	}

	relSvc := NewRelationshipServiceServer(s.storage)
	decoratedSvc, err := decorators.NewObservableRelationshipServer(relSvc, s.meterProvider)
	if err != nil {
		return err
	}

	proto.RegisterRelationshipServiceServer(s.grpcServer, decoratedSvc)

	s.Mux = r
	return nil
}

func (s *Server) ListenAndServe() error {
	go func() {
		log.Info().Msgf("HTTP server is listening on %s", s.cfg.Port)
		if err := s.httpServer.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			panic(err)
		}
	}()

	go func() {
		lis, err := net.Listen("tcp", s.cfg.GrpcServer.Port)
		if err != nil {
			panic(err)
		}

		log.Info().Msgf("gRPC server is listening on %s", s.cfg.GrpcServer.Port)
		if err := s.grpcServer.Serve(lis); err != nil {
			panic(err)
		}
	}()

	return nil
}

func (s *Server) Shutdown(ctx context.Context) error {
	if err := s.httpServer.Shutdown(ctx); err != nil {
		return err
	}

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
