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
	m "github.com/mqsrr/zylo/social-graph/internal/middleware"
	"github.com/mqsrr/zylo/social-graph/internal/mq"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"net/http"
	"time"
)

var tokenAuth *jwtauth.JWTAuth

type Server struct {
	*chi.Mux
	cfg                  *config.Config
	storage              storage.RelationshipStorage
	cache                storage.CacheStorage
	profileServiceClient ProfileService
	consumer             mq.Consumer
	httpServer           *http.Server
}

func ResponseWithJSON(w http.ResponseWriter, statusCode int, content any) {
	w.Header().Add("Content-Type", "application/json")
	w.WriteHeader(statusCode)

	if content != nil {
		jsonContent, _ := json.Marshal(content)
		w.Write(jsonContent)
	}
}

func NewServer(config *config.Config, storage storage.RelationshipStorage, cache storage.CacheStorage, consumer mq.Consumer, profileServiceClient ProfileService) *Server {
	srv := &Server{
		cfg:                  config,
		storage:              storage,
		cache:                cache,
		consumer:             consumer,
		profileServiceClient: profileServiceClient,
	}

	srv.httpServer = &http.Server{
		Addr:    config.ListeningAddress,
		Handler: srv,
	}

	return srv
}

func setupJWT(config *config.JwtConfig) {
	tokenAuth = jwtauth.New(
		"HS256",
		[]byte(config.Secret),
		nil,
		jwt.WithAcceptableSkew(5*time.Second),
		jwt.WithAudience(config.Audience),
		jwt.WithIssuer(config.Issuer))
}

func (s *Server) MountHandlers() error {
	setupJWT(s.cfg.Jwt)

	r := chi.NewRouter()

	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)
	r.Use(jwtauth.Verifier(tokenAuth))
	r.Use(jwtauth.Authenticator(tokenAuth))

	r.Route("/api/users/{userID}", func(r chi.Router) {

		r.Get("/relationships", m.MustUlidParams(s.HandleGetUserWithRelationships))

		r.Route("/followers", func(r chi.Router) {
			r.Get("/", m.MustUlidParams(s.HandleGetFollowers))
			r.Get("/me", m.MustUlidParams(s.HandleGetFollowedPeople))
			r.Post("/{followedID}", m.MustUlidParams(s.HandleFollowUser, "followedID"))
			r.Delete("/{followedID}", m.MustUlidParams(s.HandleUnfollowUser, "followedID"))
		})
		r.Route("/friends", func(r chi.Router) {
			r.Get("/", m.MustUlidParams(s.HandleGetFriends))
			r.Delete("/{friendID}", m.MustUlidParams(s.HandleRemoveFriend, "friendID"))
			r.Get("/requests", m.MustUlidParams(s.HandleGetPendingFriendRequests))
			r.Post("/requests/{receiverID}", m.MustUlidParams(s.HandleSendFriendRequest, "receiverID"))
			r.Put("/requests/{receiverID}", m.MustUlidParams(s.HandleAcceptFriendRequest, "receiverID"))
			r.Delete("/requests/{receiverID}", m.MustUlidParams(s.HandleDeclineFriendRequest, "receiverID"))
		})
		r.Route("/blocks", func(r chi.Router) {
			r.Get("/", m.MustUlidParams(s.HandleGetBlockedPeople))
			r.Post("/{blockedID}", m.MustUlidParams(s.HandleBlockUser, "blockedID"))
			r.Delete("/{blockedID}", m.MustUlidParams(s.HandleUnblockUser, "blockedID"))
		})
	})

	if err := s.HandleUserCreatedMessage(); err != nil {
		return err
	}

	if err := s.HandleUserUpdatedMessage(); err != nil {
		return err
	}

	if err := s.HandleUserDeletedMessage(); err != nil {
		return err
	}

	s.Mux = r
	return nil
}

func (s *Server) ListenAndServe() error {
	go func() {
		if err := s.httpServer.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
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

	if err := s.profileServiceClient.CloseConnection(); err != nil {
		return err
	}

	return nil
}
