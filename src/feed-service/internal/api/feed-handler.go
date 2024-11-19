package api

import (
	"net/http"
	"time"
)

type PaginatedResponse struct {
	Data        interface{} `json:"data"`
	PerPage     int         `json:"perPage"`
	HasNextPage bool        `json:"hasNextPage"`
	Next        string      `json:"next"`
}

func (s *Server) HandleGetFeed(w http.ResponseWriter, r *http.Request) error {
	requestParams, ok := r.Context().Value("requestParams").(*RequestParams)
	if !ok {
		ResponseWithJSON(w, http.StatusUnprocessableEntity, "Invalid request parameters")
		return nil
	}

	ctx := r.Context()
	postIDs, next, err := s.storage.GenerateRecommendedPostIDs(ctx, requestParams.UserID.String(), requestParams.MinLikes, requestParams.PageSize, requestParams.Cursor)
	if err != nil {
		return err
	}

	hasNextPage := len(postIDs) == requestParams.PageSize
	if len(postIDs) < 1 {
		ResponseWithJSON(w, http.StatusOK, PaginatedResponse{
			Data:        postIDs,
			PerPage:     requestParams.PageSize,
			HasNextPage: hasNextPage,
			Next:        "",
		})

		return nil
	}

	ResponseWithJSON(w, http.StatusOK, PaginatedResponse{
		Data:        postIDs,
		PerPage:     requestParams.PageSize,
		HasNextPage: hasNextPage,
		Next:        next.Format(time.RFC3339),
	})
	return nil
}
