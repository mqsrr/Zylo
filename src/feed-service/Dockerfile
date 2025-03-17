ARG GO_VERSION=1.23.6-bookworm

FROM golang:$GO_VERSION AS build
WORKDIR /usr/src/app

COPY go.mod go.sum ./
RUN go mod download && go mod verify

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o /usr/local/bin/app ./cmd/feed-service

FROM gcr.io/distroless/static-debian12

COPY --from=build /usr/local/bin/app /usr/local/bin/app
EXPOSE 50051

CMD ["/usr/local/bin/app"]