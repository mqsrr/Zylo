package main

import (
	"context"
	"github.com/joho/godotenv"
	"github.com/mqsrr/zylo/feed-service/internal/api"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/db"
	"github.com/mqsrr/zylo/feed-service/internal/mq"
	"github.com/mqsrr/zylo/feed-service/logger"
	"github.com/rs/zerolog/log"
	"os"
	"os/signal"
	"syscall"
	"time"
)

func main() {
	if err := godotenv.Load(); err != nil {
		log.Warn().Err(err).Msg("Error loading .config file, falling back to environment variables")
	}

	cfg := config.Load()
	logger.InitLogger()

	storage, err := db.NewNeo4jStorage(context.Background(), cfg.DB)
	if err != nil {
		log.Fatal().Err(err).Msg("")
		return
	}

	c, err := mq.NewConsumer(cfg.Amqp)
	if err != nil {
		log.Fatal().Err(err).Msg("")
		return
	}

	srv := api.NewServer(cfg, storage, c)
	log.Info().Msgf("Listening on %s", cfg.ListeningAddress)

	if err = srv.MountHandlers(); err != nil {
		log.Fatal().Err(err).Msg("Failed to mount handlers")
	}

	serverCtx, serverStopCtx := context.WithCancel(context.Background())

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGHUP, syscall.SIGINT, syscall.SIGTERM, syscall.SIGQUIT)
	go func() {
		<-sig
		log.Info().Msg("Shutdown signal received")

		shutdownCtx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		if err := srv.Shutdown(shutdownCtx); err != nil {
			log.Fatal().Err(err).Msg("Error during shutdown")
		}
		serverStopCtx()
	}()

	if err = srv.ListenAndServe(); err != nil {
		log.Fatal().Err(err).Msg("Server error")
	}

	<-serverCtx.Done()
	log.Info().Msg("Server stopped")
}
