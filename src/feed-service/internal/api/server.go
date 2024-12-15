package api

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/jwtauth/v5"
	"github.com/lestrrat-go/jwx/v2/jwt"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/db"
	m "github.com/mqsrr/zylo/feed-service/internal/middleware"
	"github.com/mqsrr/zylo/feed-service/internal/mq"
	"github.com/rs/zerolog/log"
	"net/http"
	"time"
)

var tokenAuth *jwtauth.JWTAuth

type Server struct {
	*chi.Mux
	config     *config.Config
	storage    db.RecommendationService
	consumer   mq.Consumer
	httpServer *http.Server
}

func ResponseWithJSON(w http.ResponseWriter, statusCode int, content any) {
	w.Header().Add("Content-Type", "application/json")
	w.WriteHeader(statusCode)

	if content != nil {
		jsonContent, _ := json.Marshal(content)
		w.Write(jsonContent)
	}
}

func NewServer(cfg *config.Config, storage db.RecommendationService, consumer mq.Consumer) *Server {
	srv := &Server{
		config:   cfg,
		storage:  storage,
		consumer: consumer,
	}

	srv.httpServer = &http.Server{
		Addr:    cfg.ListeningAddress,
		Handler: srv,
	}

	return srv
}

func setupJWT(cfg *config.JwtConfig) {
	tokenAuth = jwtauth.New(
		"HS256",
		[]byte(cfg.Secret),
		nil,
		jwt.WithAcceptableSkew(5*time.Second),
		jwt.WithAudience(cfg.Audience),
		jwt.WithIssuer(cfg.Issuer))
}

func (s *Server) MountHandlers() error {
	setupJWT(s.config.Jwt)

	r := chi.NewRouter()
	r.Use(m.ZerologMiddleware)
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Recoverer)
	r.Use(jwtauth.Verifier(tokenAuth))
	r.Use(jwtauth.Authenticator(tokenAuth))

	r.With(ValidateRequestParams).Get("/api/users/{userID}/feed", ErrHandler(s.HandleGetFeed))

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

	if err := s.storage.Shutdown(ctx); err != nil {
		log.Err(err).Msg("Database connection could not be closed")
		return err
	}

	if err := s.consumer.Shutdown(); err != nil {
		return err
	}

	return nil
}
