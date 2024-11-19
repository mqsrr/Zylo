//go:build integration

package storage_test

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"github.com/mqsrr/zylo/social-graph/internal/testutil"
	"github.com/stretchr/testify/suite"
	"github.com/testcontainers/testcontainers-go/modules/redis"
	"strings"
	"testing"
	"time"
)

type RedisCacheStorageTestSuite struct {
	suite.Suite
	redisContainer *redis.RedisContainer
	cache          storage.CacheStorage
	ctx            context.Context
}

func (suite *RedisCacheStorageTestSuite) SetupSuite() {
	redisContainer, err := testutil.StartRedisContainer()
	suite.Require().NoError(err)

	suite.ctx = context.Background()
	suite.redisContainer = redisContainer

	conn, err := suite.redisContainer.ConnectionString(suite.ctx)
	suite.Require().NoError(err)

	conn, _ = strings.CutPrefix(conn, "redis://")
	suite.cache, err = storage.NewRedisCacheStorage(suite.ctx, conn)
	suite.Require().NoError(err)
}

func (suite *RedisCacheStorageTestSuite) TearDownSuite() {
	err := suite.redisContainer.Terminate(suite.ctx)
	suite.Require().NoError(err)
}

func (suite *RedisCacheStorageTestSuite) TestHSetAndHGet() {
	key := "test_key"
	field := "test_field"
	value := map[string]string{"foo": "bar"}
	expiration := 10 * time.Second

	err := suite.cache.HSet(suite.ctx, key, field, value, expiration)
	suite.Require().NoError(err, "Failed to set value in cache")

	var retrievedValue map[string]string
	err = suite.cache.HGet(suite.ctx, key, field, &retrievedValue)
	suite.Require().NoError(err, "Failed to get value from cache")

	suite.Equal(value, retrievedValue, "Retrieved value does not match expected")
}

func (suite *RedisCacheStorageTestSuite) TestHDelete() {
	key := "delete_test_key"
	field := "delete_test_field"
	value := "to be deleted"

	err := suite.cache.HSet(suite.ctx, key, field, value, 5*time.Minute)
	suite.Require().NoError(err, "Failed to set value in cache for deletion")

	err = suite.cache.HDelete(suite.ctx, key, field)
	suite.Require().NoError(err, "Failed to delete value from cache")

	var result string
	err = suite.cache.HGet(suite.ctx, key, field, &result)
	suite.Error(err, "Expected error when getting deleted field")
	suite.Empty(result, "Deleted field should not have a value")
}

func (suite *RedisCacheStorageTestSuite) TestHDeleteAll() {
	key := "delete_all_test_key"
	field1 := "test_field_1"
	field2 := "test_field_2"
	value := "to be deleted"

	err := suite.cache.HSet(suite.ctx, key, field1, value, 5*time.Minute)
	suite.Require().NoError(err, "Failed to set first value in cache")

	err = suite.cache.HSet(suite.ctx, key, field2, value, 5*time.Minute)
	suite.Require().NoError(err, "Failed to set second value in cache")

	err = suite.cache.HDeleteAll(suite.ctx, key, "test_field_*")
	suite.Require().NoError(err, "Failed to delete all matching fields from cache")

	var result string
	err = suite.cache.HGet(suite.ctx, key, field1, &result)
	suite.Error(err, "Expected error when getting deleted field1")
	suite.Empty(result, "Deleted field1 should not have a value")

	err = suite.cache.HGet(suite.ctx, key, field2, &result)
	suite.Error(err, "Expected error when getting deleted field2")
	suite.Empty(result, "Deleted field2 should not have a value")
}

func (suite *RedisCacheStorageTestSuite) TestHSetExpiration() {
	key := "expire_test_key"
	field := "expire_test_field"
	value := "expiring value"
	expiration := 2 * time.Second

	err := suite.cache.HSet(suite.ctx, key, field, value, expiration)
	suite.Require().NoError(err, "Failed to set value in cache with expiration")

	var result string
	err = suite.cache.HGet(suite.ctx, key, field, &result)
	suite.Require().NoError(err, "Failed to get value before expiration")
	suite.Equal(value, result, "Expected value before expiration")

	time.Sleep(3 * time.Second)

	result = ""
	err = suite.cache.HGet(suite.ctx, key, field, &result)
	suite.Error(err, "Expected error when getting expired field")
	suite.Empty(result, "Expired field should not have a value")
}

func TestRedisCacheStorageSuite(t *testing.T) {
	suite.Run(t, new(RedisCacheStorageTestSuite))
}
