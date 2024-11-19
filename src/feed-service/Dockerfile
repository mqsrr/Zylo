ARG GO_VERSION=1.23.2-bullseye

ARG APP_NAME_ARG=feed-service

FROM golang:$GO_VERSION AS builder
ARG APP_NAME_ARG
WORKDIR /usr/src/${APP_NAME_ARG}

COPY go.mod go.sum ./
RUN go mod download && go mod verify

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -v -o /usr/local/bin/${APP_NAME_ARG} ./cmd/${APP_NAME_ARG}/

FROM debian:bullseye-slim
ARG APP_NAME_ARG
ENV APP_NAME=${APP_NAME_ARG}
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/${APP_NAME_ARG} /usr/local/bin/${APP_NAME_ARG}

EXPOSE 8080
CMD ${APP_NAME}