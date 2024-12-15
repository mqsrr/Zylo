package middleware

import (
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/oklog/ulid/v2"
	"github.com/rs/zerolog/log"
	"net/http"
	"time"
)

type ErrHandlerFunc func(w http.ResponseWriter, r *http.Request) error

func ErrHandler(h ErrHandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		err := h(w, r)
		if err != nil {
			w.Header().Add("Content-Type", "application/json")
			w.WriteHeader(http.StatusInternalServerError)

			log.Error().Err(err).Msg("")
			return
		}
	}
}

func MustUlidParams(h ErrHandlerFunc, params ...string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		params = append(params, "userID")
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
