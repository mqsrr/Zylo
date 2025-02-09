package middleware

import (
	"errors"
	"fmt"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/rs/zerolog/log"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	semconv "go.opentelemetry.io/otel/semconv/v1.27.0"
	"go.opentelemetry.io/otel/trace"
	"net/http"
	"time"
)

type ErrHandlerFunc func(w http.ResponseWriter, r *http.Request) error

func ErrHandler(h ErrHandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var errResponse *types.ProblemResponse
		rw := middleware.NewWrapResponseWriter(w, r.ProtoMajor)

		err := h(rw, r)
		if err != nil {
			defer func() {
				span := trace.SpanFromContext(r.Context())
				span.SetStatus(codes.Error, err.Error())
				span.RecordError(err)

				span.SetAttributes(semconv.HTTPResponseStatusCode(rw.Status()))
			}()

			w.Header().Add("Content-Type", "application/json")
			if errors.As(err, &errResponse) {
				if errResponse.StatusCode == http.StatusInternalServerError {
					log.Error().Err(err).Msg("")
				}

				types.WriteProblemResponse(rw, errResponse)
				return
			}

			rw.WriteHeader(http.StatusInternalServerError)
			return
		}
	}
}

func Instrumented(traceProvider trace.TracerProvider, meterProvider metric.MeterProvider) func(http.Handler) http.Handler {

	tracer := traceProvider.Tracer("api")
	meter := meterProvider.Meter("api")

	requestCounter, _ := meter.Int64Counter("relationships_api_request_total", metric.WithDescription("Total requests to RelationshipStorage"))
	requestLatency, _ := meter.Float64Histogram("relationships_api_duration_seconds",
		metric.WithDescription("Latency of RelationshipStorage"),
		metric.WithExplicitBucketBoundaries(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0))

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			methodName := fmt.Sprintf("%s %s", r.Method, r.URL.Path)
			ctx, span := tracer.Start(r.Context(), methodName,
				trace.WithSpanKind(trace.SpanKindServer),
				trace.WithAttributes(
					semconv.HTTPRequestMethodKey.String(r.Method),
					semconv.HTTPRequestMethodOriginal(r.Method),
					semconv.ServerAddress("127.0.0.0"),
					semconv.ServerPort(8080),
					semconv.HTTPRoute(r.URL.Path)),
			)
			requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", methodName)))

			defer span.End()
			rw := middleware.NewWrapResponseWriter(w, r.ProtoMajor)
			startTime := time.Now()

			next.ServeHTTP(rw, r.WithContext(ctx))
			duration := time.Since(startTime).Seconds()

			requestLatency.Record(ctx, duration, metric.WithAttributes(attribute.String("method", methodName)))
			span.SetAttributes(semconv.HTTPResponseStatusCode(rw.Status()))
		})
	}
}

func MustUlidParams(h ErrHandlerFunc, params ...string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		params = append(params, "id")
		for _, param := range params {
			_, err := ulid.Parse(chi.URLParam(r, param))
			if err != nil {
				w.Header().Add("Content-Type", "application/json")
				w.WriteHeader(http.StatusBadRequest)

				w.Write([]byte("ID param is not a ULID type"))
			}
		}
		ErrHandler(h).ServeHTTP(w, r)
	}
}

func ZerologMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		rw := middleware.NewWrapResponseWriter(w, r.ProtoMajor)

		log.Info().
			Str("method", r.Method).
			Str("path", r.URL.Path).
			Str("remote_addr", r.RemoteAddr).
			Msg("request started")

		next.ServeHTTP(rw, r)

		log.Info().
			Str("method", r.Method).
			Str("path", r.URL.Path).
			Int("status", rw.Status()).
			Str("status_text", http.StatusText(rw.Status())).
			Dur("duration", time.Since(start)).
			Msg("request completed")
	})
}
