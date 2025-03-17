package main

import (
	"context"
	"github.com/joho/godotenv"
	"github.com/mqsrr/zylo/feed-service/internal/api"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/logger"
	"github.com/mqsrr/zylo/feed-service/internal/middleware"
	"github.com/mqsrr/zylo/feed-service/internal/mq"
	"github.com/mqsrr/zylo/feed-service/internal/storage"
	"github.com/rs/zerolog/log"
	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
	"google.golang.org/grpc"
	"os"
	"os/signal"
	"syscall"
)

func main() {
	if err := godotenv.Load(); err != nil {
		log.Warn().Err(err).Msg("Error loading .config file, falling back to environment variables")
	}

	logger.InitLogger()
	ctx, serverStopCtx := context.WithCancel(context.Background())

	cfg, err := config.Load()
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
	}

	tp, err := api.InitTracerProvider(ctx, cfg.OtelCollector)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	mp, err := api.InitMeterProvider(ctx, cfg.OtelCollector)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	lp, err := api.InitLoggerProvider(ctx, cfg.OtelCollector)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	driver, err := storage.NewNeo4jDriverWithContext(ctx, cfg.DB)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	c, err := mq.NewConsumer(cfg.Amqp)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
		return
	}

	grpcServer := grpc.NewServer(
		grpc.StatsHandler(otelgrpc.NewServerHandler(otelgrpc.WithTracerProvider(tp))),
		grpc.UnaryInterceptor(middleware.UnaryLoggingInterceptor()))

	feedServer, err := api.NewFeedServiceServer(driver, tp, mp)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
	}

	srv, err := api.NewServer(ctx, cfg, c, grpcServer, tp, mp, lp, driver)
	if err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("")
	}

	if err = srv.MountHandlers(feedServer); err != nil {
		log.Fatal().Ctx(ctx).Err(err).Msg("Failed to mount handlers")
	}

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGHUP, syscall.SIGINT, syscall.SIGTERM, syscall.SIGQUIT)
	go func() {
		<-sig
		log.Info().Ctx(ctx).Msg("Shutdown signal received")

		if err := srv.Shutdown(ctx); err != nil {
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
