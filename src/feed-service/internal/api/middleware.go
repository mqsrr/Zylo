package api

import (
	"context"
	"github.com/go-chi/chi/v5"
	"github.com/oklog/ulid"
	"github.com/rs/zerolog/log"
	"net/http"
	"strconv"
	"time"
)

type RequestParams struct {
	UserID   ulid.ULID
	Cursor   *time.Time
	PageSize int
	MinLikes int
}

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

func ValidateRequestParams(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		userIDStr := chi.URLParam(r, "userID")
		userID, err := ulid.Parse(userIDStr)
		if err != nil {
			ResponseWithJSON(w, http.StatusBadRequest, "Invalid user ID")
			return
		}

		queryParams := r.URL.Query()
		pageSizeStr := queryParams.Get("pageSize")

		pageSize, err := parsePageSize(pageSizeStr)
		if err != nil {
			ResponseWithJSON(w, http.StatusBadRequest, "Invalid page size")
			return
		}

		cursorStr := queryParams.Get("next")
		cursor, err := parseCursor(cursorStr)
		if err != nil {
			ResponseWithJSON(w, http.StatusBadRequest, "Invalid cursor format. The server supports RFC3339(\"2006-01-02T15:04:05Z07:00\") time format")
			return
		}

		minLikesStr := queryParams.Get("minLikes")
		minLikes, err := parseMinLikes(minLikesStr)
		if err != nil {
			ResponseWithJSON(w, http.StatusBadRequest, "Invalid min likes size")
			return
		}

		ctx := context.WithValue(r.Context(), "requestParams", &RequestParams{
			UserID:   userID,
			PageSize: pageSize,
			Cursor:   cursor,
			MinLikes: minLikes,
		})

		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

func parsePageSize(pageSizeStr string) (int, error) {
	if pageSizeStr == "" {
		return 10, nil
	}
	pageSize, err := strconv.Atoi(pageSizeStr)
	if err != nil || pageSize <= 0 {
		return 0, err
	}
	return pageSize, nil
}

func parseMinLikes(minLikes string) (int, error) {
	if minLikes == "" {
		return 10, nil
	}
	pageSize, err := strconv.Atoi(minLikes)
	if err != nil || pageSize <= 0 {
		return 0, err
	}
	return pageSize, nil
}

func parseCursor(cursorStr string) (*time.Time, error) {
	if cursorStr == "" {
		return nil, nil
	}
	cursorTime, err := time.Parse(time.RFC3339, cursorStr)
	if err != nil {
		return nil, err
	}

	return &cursorTime, nil
}
