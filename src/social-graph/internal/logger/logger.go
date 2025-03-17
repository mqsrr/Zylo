package logger

import (
	"context"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"go.opentelemetry.io/otel/trace"
)

func InitLogger() {
	zerolog.SetGlobalLevel(zerolog.InfoLevel)
	log.Logger = log.With().Caller().Logger().Hook(RequestHook{})
}

type TracingHook struct{}

func (h TracingHook) Run(e *zerolog.Event, level zerolog.Level, msg string) {
	ctx := e.GetCtx()
	if spanId := getSpanIdFromContext(ctx); spanId != "" {
		e.Str("span.id", spanId)
	}

	if traceId := getTraceIdFromContext(ctx); traceId != "" {
		e.Str("trace.id", traceId)
	}
}

func getSpanIdFromContext(ctx context.Context) string {
	span := trace.SpanFromContext(ctx)
	if span == nil {
		return ""
	}

	sc := span.SpanContext()
	if !sc.IsValid() {
		return ""
	}
	return sc.SpanID().String()
}

func getTraceIdFromContext(ctx context.Context) string {
	span := trace.SpanFromContext(ctx)
	if span == nil {
		return ""
	}

	sc := span.SpanContext()
	if !sc.IsValid() {
		return ""
	}
	return sc.TraceID().String()
}

type RequestHook struct{}

func (h RequestHook) Run(e *zerolog.Event, level zerolog.Level, msg string) {
	ctx := e.GetCtx()
	if requestId := getRequestIdFromContext(ctx); requestId != "" {
		e.Str("http.request.id", requestId)
	}
}

func getRequestIdFromContext(ctx context.Context) string {
	if requestId := middleware.GetReqID(ctx); requestId != "" {
		return requestId
	}

	if requestId := ctx.Value("x-request-id"); requestId != nil {
		return requestId.(string)
	}
	return ""
}
