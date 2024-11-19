//go:build unit

package logger

import (
	"bytes"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestInitLogger_Development(t *testing.T) {
	config.DefaultConfig = &config.PublicConfig{Environment: "Development"}

	InitLogger()

	var buf bytes.Buffer
	log.Logger = log.Output(&buf)

	log.Info().Msg("development log message")
	assert.Contains(t, buf.String(), "development log message")
	assert.Contains(t, buf.String(), "logger_test.go")
}

func TestInitLogger_Production(t *testing.T) {
	config.DefaultConfig = &config.PublicConfig{Environment: "Production"}

	InitLogger()

	var buf bytes.Buffer
	log.Logger = log.Output(&buf)

	log.Info().Msg("production log message")
	assert.Contains(t, buf.String(), "production log message")
	assert.NotContains(t, buf.String(), "caller=logger_test.go")
}

func TestLogLevel(t *testing.T) {
	config.DefaultConfig = &config.PublicConfig{Environment: "Development"}
	InitLogger()
	assert.Equal(t, zerolog.DebugLevel, zerolog.GlobalLevel())

	config.DefaultConfig = &config.PublicConfig{Environment: "Production"}
	InitLogger()
	assert.Equal(t, zerolog.InfoLevel, zerolog.GlobalLevel())
}
