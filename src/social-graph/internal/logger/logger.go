package logger

import (
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"os"
)

func InitLogger() {
	zerolog.SetGlobalLevel(zerolog.InfoLevel)

	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix
	log.Logger = log.With().Caller().Logger()

	ENVIRONMENT := config.DefaultConfig.Environment
	if ENVIRONMENT == "Development" {
		log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stderr})
	}
}
