package main

import (
	"context"
	"github.com/joho/godotenv"
	"github.com/mqsrr/zylo/social-graph/internal/api"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/logger"
	"github.com/mqsrr/zylo/social-graph/internal/mq"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"github.com/rs/zerolog/log"
)

func main() {
	if err := godotenv.Load(); err != nil {
		log.Warn().Err(err).Msg("Error loading .env file, falling back to environment variables")
	}

	client, err := config.CreateKeyVaultClient()
	if err != nil {
		log.Fatal().Err(err).Msg("")
		return
	}

	cfg := config.Load(client)
	logger.InitLogger()

	ctx := context.Background()
	db, err := storage.NewNeo4jStorage(ctx, cfg.DB.Uri, cfg.DB.Username, cfg.DB.Password)
	if err != nil {
		log.Fatal().Err(err).Msg("")
		return
	}

	r, err := storage.NewRedisCacheStorage(ctx, cfg.Redis.ConnectionString)
	if err != nil {
		log.Fatal().Err(err).Msg("")
		return
	}

	c, err := mq.NewConsumer(cfg.Amqp)
	if err != nil {
		log.Fatal().Err(err).Msg("")
		return
	}

	grpcClient, err := api.NewProfileService(cfg.Grpc)
	if err != nil {
		log.Fatal().Err(err).Msg("")
		return
	}
	defer grpcClient.CloseConnection()

	srv := api.NewServer(cfg, db, r, c, grpcClient)
	log.Info().Msgf("Listening on %s", cfg.ListeningAddress)

	if err = srv.MountHandlers(); err != nil {
		log.Fatal().Err(err).Msg("")
	}

	if err = srv.ListenAndServe(); err != nil {
		log.Fatal().Err(err).Msg("")
	}
}
