package middleware

import (
	"errors"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/rs/zerolog/log"
	"net/http"
	"time"
)

type ErrHandlerFunc func(w http.ResponseWriter, r *http.Request) error

func ErrHandler(h ErrHandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var errResponse *types.ProblemResponse
		err := h(w, r)
		if err != nil {
			w.Header().Add("Content-Type", "application/json")

			if errors.As(err, &errResponse) {
				if errResponse.StatusCode == http.StatusInternalServerError {
					log.Error().Err(err).Msg("")
				}

				types.WriteProblemResponse(w, errResponse)
				return
			}

			w.WriteHeader(http.StatusInternalServerError)
			return
		}
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
