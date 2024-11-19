//go:build integration

package testutil

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/api"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/mq"
	proto "github.com/mqsrr/zylo/social-graph/internal/protos/github.com/mqsrr/zylo/social-graph/proto/mocks"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	mocks "github.com/mqsrr/zylo/social-graph/internal/storage/mocks"
	"github.com/testcontainers/testcontainers-go/modules/neo4j"
	"github.com/testcontainers/testcontainers-go/modules/rabbitmq"
	"github.com/testcontainers/testcontainers-go/modules/redis"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
)

var (
	Neo4jContainer      *neo4j.Neo4jContainer
	RabbitmqContainer   *rabbitmq.RabbitMQContainer
	RedisContainer      *redis.RedisContainer
	once                sync.Once
	Ctx                 context.Context
	Server              *api.Server
	HttpTestServer      *httptest.Server
	RelationshipStorage storage.RelationshipStorage
	CacheStorage        storage.CacheStorage
	MqConsumer          mq.Consumer
	ProfileService      *proto.MockUserProfileServiceClient
	Cfg                 *config.Config
)

func SetupTestServer(t *testing.T) {
	SetupRelationshipStorage()
	SetupCacheStorage()
	SetupRabbitMqConsumer()
	ProfileService = proto.NewMockUserProfileServiceClient(t)

	Cfg = &config.Config{
		Jwt: &config.JwtConfig{
			Secret:   "testsecret",
			Audience: "testaudience",
			Issuer:   "testissuer",
		},
	}

	Server = api.NewServer(Cfg, RelationshipStorage, CacheStorage, MqConsumer, ProfileService)
	err := Server.MountHandlers()

	if err != nil {
		panic(err)
	}
	HttpTestServer = httptest.NewServer(Server)
}

func TearDown() error {
	if HttpTestServer != nil {
		HttpTestServer.Close()
	}

	return CleanupSharedContainers()
}

func StartNeo4jContainer() (*neo4j.Neo4jContainer, error) {
	return neo4j.Run(context.Background(), "neo4j:4.4", neo4j.WithAdminPassword("test"))
}
func StartRedisContainer() (*redis.RedisContainer, error) {
	return redis.Run(context.Background(), "redis:alpine")
}
func StartRabbitMqContainer() (*rabbitmq.RabbitMQContainer, error) {
	return rabbitmq.Run(context.Background(), "rabbitmq:management-alpine")
}

func StartTestContainers() error {
	var err error
	once.Do(func() {
		Ctx = context.Background()
		Neo4jContainer, err = StartNeo4jContainer()
		if err != nil {
			return
		}

		RabbitmqContainer, err = rabbitmq.Run(Ctx, "rabbitmq:management-alpine")
		if err != nil {
			return
		}

		RedisContainer, err = redis.Run(Ctx, "redis:alpine")
		if err != nil {
			return
		}
	})
	return err
}

func CleanupSharedContainers() error {
	var err error
	if Neo4jContainer != nil {
		err = Neo4jContainer.Terminate(Ctx)
	}
	if RabbitmqContainer != nil {
		err = RabbitmqContainer.Terminate(Ctx)
	}
	if RedisContainer != nil {
		err = RedisContainer.Terminate(Ctx)
	}
	return err
}

func SetupRelationshipStorage() {
	neo4jURI, err := Neo4jContainer.BoltUrl(Ctx)

	RelationshipStorage, err = storage.NewNeo4jStorage(Ctx, neo4jURI, "neo4j", "test")
	if err != nil {
		panic(err)
	}
}

func SetupCacheStorage() {
	conn, err := RedisContainer.ConnectionString(Ctx)
	conn, _ = strings.CutPrefix(conn, "redis://")

	CacheStorage, err = storage.NewRedisCacheStorage(Ctx, conn)
	if err != nil {
		panic(err)
	}
}

func SetupRabbitMqConsumer() {
	rabbitmqURI, err := RabbitmqContainer.AmqpURL(Ctx)
	if err != nil {
		panic(err)
	}
	rabbitMqConfig := &config.RabbitmqConfig{
		AmqpURI:  rabbitmqURI,
		ConnTag:  "test",
		ConnName: "test-conn",
	}

	MqConsumer, err = mq.NewConsumer(rabbitMqConfig)
	if err != nil {
		panic(err)
	}

}
