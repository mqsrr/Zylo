//go:build unit

package middleware

import (
	"errors"
	"github.com/go-chi/chi/v5"
	"github.com/oklog/ulid/v2"
	"github.com/stretchr/testify/assert"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestErrHandler_Error(t *testing.T) {
	handler := ErrHandler(func(w http.ResponseWriter, r *http.Request) error {
		return errors.New("mock error")
	})

	req := httptest.NewRequest(http.MethodGet, "/", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusInternalServerError, rec.Code)
	assert.Equal(t, "application/json", rec.Header().Get("Content-Type"))
}

func TestErrHandler_NoError(t *testing.T) {
	handler := ErrHandler(func(w http.ResponseWriter, r *http.Request) error {
		w.Write([]byte("no error"))
		return nil
	})

	req := httptest.NewRequest(http.MethodGet, "/", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
	assert.Equal(t, "no error", rec.Body.String())
}

func TestMustUlidParams_ValidULID(t *testing.T) {
	r := chi.NewRouter()
	r.HandleFunc("/users/{userID}", MustUlidParams(func(w http.ResponseWriter, r *http.Request) error {
		w.Write([]byte("valid ULID"))
		return nil
	}, "userID"))

	userID := ulid.Make().String()
	req := httptest.NewRequest(http.MethodGet, "/users/"+userID, nil)
	rec := httptest.NewRecorder()

	r.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
	assert.Equal(t, "valid ULID", rec.Body.String())
}

func TestMustUlidParams_InvalidULID(t *testing.T) {
	r := chi.NewRouter()
	r.HandleFunc("/users/{userID}", MustUlidParams(func(w http.ResponseWriter, r *http.Request) error {
		return nil
	}, "userID"))

	req := httptest.NewRequest(http.MethodGet, "/users/invalid-ulid", nil)
	rec := httptest.NewRecorder()

	r.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusBadRequest, rec.Code)
	assert.Equal(t, "application/json", rec.Header().Get("Content-Type"))
	assert.Contains(t, rec.Body.String(), "ID param is not a ULID type")
}
