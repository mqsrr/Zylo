package main

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/api"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/db"
	"github.com/mqsrr/zylo/feed-service/internal/mq"
	"github.com/mqsrr/zylo/feed-service/logger"
	"github.com/rs/zerolog/log"
)

func main() {
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

	err = srv.MountHandler()
	if err != nil {
		log.Fatal().Err(err).Msg("Could not mount the handlers")
	}

	err = srv.ListenAndServe()
	if err != nil {
		log.Fatal().Err(err).Msg("Could not start a server")
	}
}
