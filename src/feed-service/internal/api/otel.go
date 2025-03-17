package api

import (
	"context"
	"github.com/mqsrr/zylo/feed-service/internal/config"
	"github.com/mqsrr/zylo/feed-service/internal/logger"
	zerolog "github.com/rs/zerolog/log"
	"go.opentelemetry.io/contrib/bridges/otelzerolog"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc"
	"go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetricgrpc"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/log/global"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/log"
	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/resource"
	"go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.27.0"
)

func InitLoggerProvider(ctx context.Context, cfg *config.OtelCollector) (*log.LoggerProvider, error) {
	exporter, err := otlploggrpc.New(ctx,
		otlploggrpc.WithEndpoint(cfg.Address),
		otlploggrpc.WithInsecure())

	if err != nil {
		return nil, err
	}
	processor := log.NewBatchProcessor(exporter)
	provider := log.NewLoggerProvider(log.WithProcessor(processor), log.WithResource(resource.NewWithAttributes(
		semconv.SchemaURL,
		semconv.ServiceNameKey.String("feed-service"),
	)))

	hook := otelzerolog.NewHook("feed-service", otelzerolog.WithLoggerProvider(provider))

	zerolog.Logger = zerolog.Logger.Hook(hook)
	global.SetLoggerProvider(provider)
	return provider, nil
}

func InitTracerProvider(ctx context.Context, cfg *config.OtelCollector) (*trace.TracerProvider, error) {
	exporter, err := otlptracegrpc.New(ctx,
		otlptracegrpc.WithEndpoint(cfg.Address),
		otlptracegrpc.WithInsecure(),
	)
	if err != nil {
		return nil, err
	}

	tp := trace.NewTracerProvider(
		trace.WithBatcher(exporter),
		trace.WithResource(resource.NewWithAttributes(
			semconv.SchemaURL,
			semconv.ServiceNameKey.String("feed-service"),
		)),
	)

	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.TraceContext{})
	zerolog.Logger = zerolog.Logger.Hook(logger.TracingHook{})
	return tp, nil
}

func InitMeterProvider(ctx context.Context, cfg *config.OtelCollector) (*metric.MeterProvider, error) {
	exporter, err := otlpmetricgrpc.New(ctx,
		otlpmetricgrpc.WithEndpoint(cfg.Address),
		otlpmetricgrpc.WithInsecure(),
	)
	if err != nil {
		return nil, err
	}

	reader := metric.NewPeriodicReader(exporter)

	mp := metric.NewMeterProvider(
		metric.WithReader(reader),
		metric.WithResource(resource.NewWithAttributes(
			semconv.SchemaURL,
			semconv.ServiceNameKey.String("feed-service"),
		)),
	)
	otel.SetMeterProvider(mp)
	return mp, nil
}
