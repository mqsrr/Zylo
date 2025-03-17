package middleware

import (
	"context"
	"github.com/rs/zerolog/log"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	"time"
)

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
