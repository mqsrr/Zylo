package middleware

import (
	"context"
	"errors"
	"fmt"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/rs/zerolog/log"
	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	semconv "go.opentelemetry.io/otel/semconv/v1.27.0"
	"go.opentelemetry.io/otel/trace"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
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

func OtelMiddleware(meterProvider metric.MeterProvider) func(http.Handler) http.Handler {
	meter := meterProvider.Meter("social-graph")

	requestCounter, _ := meter.Int64Counter("social_graph_relationships_api_request_count", metric.WithDescription("Total requests to Relationship API"))
	requestLatency, _ := meter.Float64Histogram("social_graph_relationships_api_duration",
		metric.WithDescription("Duration of requests to Relationship API"),
		metric.WithExplicitBucketBoundaries(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0))
	containerId, err := config.GetContainerID()
	if err != nil {
		containerId = "0.0.0.0"
	}
	attributes := []attribute.KeyValue{
		attribute.String("service", "social-graph"),
		attribute.String("instance", fmt.Sprintf("%s:%s", containerId, config.DefaultConfig.Port)),
		attribute.String("env", config.DefaultConfig.Environment),
	}

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ctx := r.Context()
			methodName := fmt.Sprintf("%s %s", r.Method, r.URL.Path)

			if requestId := r.Header.Get("x-request-id"); requestId != "" {
				w.Header().Set("x-request-id", requestId)
			}

			handler := otelhttp.NewHandler(next, methodName)
			requestCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("method", methodName)))

			rw := middleware.NewWrapResponseWriter(w, r.ProtoMajor)
			startTime := time.Now()

			handler.ServeHTTP(rw, r.WithContext(ctx))
			duration := time.Since(startTime).Seconds()

			status := "success"
			if rw.Status() > 499 {
				status = "error"
			}

			requestCounter.Add(ctx, 1,
				metric.WithAttributes(
					attribute.String("method", methodName),
					attribute.String("status", status)),
				metric.WithAttributeSet(attribute.NewSet(attributes...)))

			requestLatency.Record(ctx, duration, metric.WithAttributes(
				attribute.String("method", "GetUserRelationships"),
				attribute.String("status", status)),
				metric.WithAttributeSet(attribute.NewSet(attributes...)))
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

func RequestLogger(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		rw := middleware.NewWrapResponseWriter(w, r.ProtoMajor)
		log.Info().
			Ctx(ctx).
			Str("method", r.Method).
			Str("path", r.URL.Path).
			Str("remote.addr", r.RemoteAddr).
			Msg("request started")
		start := time.Now()

		next.ServeHTTP(rw, r)
		stop := time.Since(start)

		log.Info().
			Ctx(ctx).
			Str("method", r.Method).
			Str("path", r.URL.Path).
			Str("remote.addr", r.RemoteAddr).
			Int("status", rw.Status()).
			Str("status.text", http.StatusText(rw.Status())).
			Dur("duration", stop).
			Msg("request completed")
	})
}

func getRequestIDFromMetadata(ctx context.Context) string {
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return ""
	}
	values := md.Get("x-request-id")
	if len(values) > 0 {
		return values[0]
	}
	return ""
}

func UnaryLoggingInterceptor() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		if requestID := getRequestIDFromMetadata(ctx); requestID != "" {
			ctx = context.WithValue(ctx, "x-request-id", requestID)
		}

		log.Info().
			Ctx(ctx).
			Str("method", info.FullMethod).
			Msg("RPC started")

		startTime := time.Now()
		resp, err := handler(ctx, req)
		duration := time.Since(startTime)

		if err != nil {
			log.Error().
				Ctx(ctx).
				Str("method", info.FullMethod).
				Dur("duration", duration).
				Err(err).
				Msg("RPC failed")
			return resp, err
		}

		log.Info().
			Ctx(ctx).
			Str("method", info.FullMethod).
			Dur("duration", duration).
			Msg("RPC completed")
		return resp, err
	}
}
