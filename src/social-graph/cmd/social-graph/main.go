package main

import (
	"context"
	"github.com/joho/godotenv"
	"github.com/mqsrr/zylo/social-graph/internal/api"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/decorators"
	"github.com/mqsrr/zylo/social-graph/internal/logger"
	"github.com/mqsrr/zylo/social-graph/internal/middleware"
	"github.com/mqsrr/zylo/social-graph/internal/mq"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"github.com/rs/zerolog/log"
	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
	"google.golang.org/grpc"
	"os"
	"os/signal"
	"syscall"
	"time"
)

func main() {
	if err := godotenv.Load(); err != nil {
		log.Warn().Err(err).Msg("Error loading .env file, falling back to environment variables")
	}
	logger.InitLogger()
	ctx, serverStopCtx := context.WithCancel(context.Background())

	cfg, err := config.Load()
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	tp, err := api.InitTracer(ctx, cfg.OtelCollector)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	mp, err := api.InitMeter(ctx, cfg.OtelCollector)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	lp, err := api.InitLogger(ctx, cfg.OtelCollector)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	grpcServer := grpc.NewServer(
		grpc.StatsHandler(otelgrpc.NewServerHandler(otelgrpc.WithTracerProvider(tp))),
		grpc.UnaryInterceptor(middleware.UnaryLoggingInterceptor()))

	r, err := storage.NewRedisCacheStorage(ctx, cfg.Redis.ConnectionString)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	observableRedis, err := decorators.NewObservableRedisStorage(r, tp, mp)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	c, err := mq.NewConsumer(cfg.Amqp)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	db, err := storage.NewNeo4jStorage(ctx, cfg.DB.Uri, cfg.DB.Username, cfg.DB.Password)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}
	observableDb, err := decorators.NewObservableNeo4jStorage(db, tp, mp)
	cachedDb := decorators.NewCachedNeo4jStorage(observableDb, cfg.Redis, observableRedis)

	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
	}

	srv := api.NewServer(cfg, cachedDb, grpcServer, observableRedis, c, tp, mp, lp)

	if err = srv.MountHandlers(); err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("Failed to mount handlers")
	}

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGHUP, syscall.SIGINT, syscall.SIGTERM, syscall.SIGQUIT)
	go func() {
		<-sig
		log.Info().Ctx(ctx).Msg("Shutdown signal received")

		shutdownCtx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		if err := srv.Shutdown(shutdownCtx); err != nil {
			log.Fatal().Ctx(ctx).Err(err).Msg("Error during shutdown")
		}
		serverStopCtx()
	}()

	if err = srv.ListenAndServe(); err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("Server error")
	}

	<-ctx.Done()
	log.Info().Ctx(ctx).Msg("Server stopped")
}
