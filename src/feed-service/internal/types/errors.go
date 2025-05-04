package types

import (
	"errors"
	"fmt"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"net/http"
)

type ProblemResponse struct {
	Title      string `json:"title"`
	Detail     string `json:"detail"`
	StatusCode int    `json:"status"`
	TraceID    string `json:"traceId,omitempty"`
	ErrType    string `json:"type"`
	Err        error  `json:"-"`
}

func (e *ProblemResponse) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("%s: %v", e.Detail, e.Err)
	}
	return e.Detail
}

func NewAppError(title, detail, errType string, statusCode int, err error) *ProblemResponse {
	return &ProblemResponse{
		Title:      title,
		Detail:     detail,
		StatusCode: statusCode,
		ErrType:    errType,
		Err:        err,
	}
}

func NewInternalError(err error) *ProblemResponse {
	return NewAppError("Internal Server Error", "Unexpected error occured", "https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.1", http.StatusInternalServerError, err)
}

func NewNotFound(detail string) *ProblemResponse {
	return NewAppError("Not Found", detail, "https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.4", http.StatusNotFound, errors.New(detail))
}

func NewBadRequest(detail string) *ProblemResponse {
	return NewAppError("Bad Request", detail, "https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.1", http.StatusBadRequest, errors.New(detail))
}

func NewBadRequestErr(detail string, err error) *ProblemResponse {
	return NewAppError("Bad Request", detail, "https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.1", http.StatusBadRequest, err)
}

func GrpcError(err error) error {
	var appErr *ProblemResponse
	if errors.As(err, &appErr) {
		return status.Errorf(grpcCodeFromStatus(appErr.StatusCode), appErr.Detail)
	}

	return status.Errorf(codes.Internal, "unexpected error")
}

func grpcCodeFromStatus(httpStatus int) codes.Code {
	switch httpStatus {
	case http.StatusBadRequest:
		return codes.InvalidArgument
	case http.StatusUnauthorized:
		return codes.Unauthenticated
	case http.StatusNotFound:
		return codes.NotFound
	case http.StatusInternalServerError:
		return codes.Internal
	default:
		return codes.Unknown
	}
}
