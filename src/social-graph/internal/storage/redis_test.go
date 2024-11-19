//go:build unit

package storage

import (
	"context"
	"encoding/json"
	"errors"
	mocks "github.com/mqsrr/zylo/social-graph/internal/storage/mocks"
	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"testing"
	"time"
)

type TestCaseType int

const (
	Success TestCaseType = iota
	CacheFailure
	ProvidedValueIsNull
	HScanFailure
	NotFound
)

var mapCaseName = map[TestCaseType]string{
	Success:             "Success",
	CacheFailure:        "Cache Failure",
	ProvidedValueIsNull: "Provided Value Is Null",
	HScanFailure:        "HScan Failure",
	NotFound:            "Not Found",
}

type RedisCacheStorageTestSuite struct {
	suite.Suite
	mockRedis        *mocks.MockRedisClient
	cache            *RedisCacheStorage
	testTypeHandlers map[TestCaseType]func()
}

type TestConfig struct {
	caseType    TestCaseType
	expectedErr bool
}

func (s *RedisCacheStorageTestSuite) SetupTest() {
	s.mockRedis = mocks.NewMockRedisClient(s.T())
	s.cache = &RedisCacheStorage{redis: s.mockRedis}
	s.testTypeHandlers = make(map[TestCaseType]func())
}

func (s *RedisCacheStorageTestSuite) On(testType TestCaseType, handler func()) *RedisCacheStorageTestSuite {
	s.testTypeHandlers[testType] = handler
	return s
}

func (s *RedisCacheStorageTestSuite) ExecuteScenario(tc TestConfig, testFunc func() error) {
	s.testTypeHandlers[tc.caseType]()
	err := testFunc()
	s.mockRedis.AssertExpectations(s.T())

	if tc.expectedErr {
		s.Require().Error(err)
		return
	}

	s.Require().NoError(err)
}

func (s *RedisCacheStorageTestSuite) TestHSet() {
	t := s.T()
	ctx := context.Background()
	key := "testKey"
	field := "testField"
	value := "testValue"
	expire := time.Minute

	valueJson, _ := json.Marshal(value)
	tests := []TestConfig{
		{caseType: Success, expectedErr: false},
		{caseType: CacheFailure, expectedErr: true},
		{caseType: ProvidedValueIsNull, expectedErr: false},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					intCmd := redis.NewIntCmd(ctx)
					intSliceCmd := redis.NewIntSliceCmd(ctx)

					s.mockRedis.EXPECT().HSet(ctx, key, field, string(valueJson)).Return(intCmd).Once()
					s.mockRedis.EXPECT().HExpire(ctx, key, expire, field).Return(intSliceCmd).Once()
				}).
				On(CacheFailure, func() {
					intCmd := redis.NewIntCmd(ctx)
					intCmd.SetErr(errors.New("redis Error"))

					s.mockRedis.EXPECT().HSet(ctx, key, field, string(valueJson)).Return(intCmd).Once()
				}).
				On(ProvidedValueIsNull, func() {

				})

			s.ExecuteScenario(tc, func() error {
				if tc.caseType == ProvidedValueIsNull {
					return s.cache.HSet(ctx, key, field, nil, expire)
				}

				return s.cache.HSet(ctx, key, field, value, expire)
			})
		})
	}
}

func (s *RedisCacheStorageTestSuite) TestHGet() {
	t := s.T()
	ctx := context.Background()
	key := "testKey"
	field := "testField"
	expectedValue := "testValue"
	var result string

	expectedJSON, _ := json.Marshal(expectedValue)
	tests := []TestConfig{
		{caseType: Success, expectedErr: false},
		{caseType: CacheFailure, expectedErr: true},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.On(Success, func() {
				s.mockRedis.EXPECT().HGet(ctx, key, field).Return(redis.NewStringResult(string(expectedJSON), nil)).Once()
			}).On(CacheFailure, func() {
				s.mockRedis.EXPECT().HGet(ctx, key, field).Return(redis.NewStringResult("", errors.New("get error"))).Once()
			})

			s.ExecuteScenario(tc, func() error {
				return s.cache.HGet(ctx, key, field, &result)
			})

			if tc.caseType == Success {
				assert.Equal(s.T(), expectedValue, result)
			}
		})

	}
}

func (s *RedisCacheStorageTestSuite) TestHDelete() {
	t := s.T()
	ctx := context.Background()
	key := "testKey"
	fields := []string{"field1", "field2"}

	tests := []TestConfig{
		{caseType: Success, expectedErr: false},
		{caseType: CacheFailure, expectedErr: true},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.On(Success, func() {
				intCmd := redis.NewIntCmd(ctx)

				s.mockRedis.EXPECT().HDel(ctx, key, fields[0], fields[1]).Return(intCmd).Once()
			}).On(CacheFailure, func() {
				intCmd := redis.NewIntCmd(ctx)
				intCmd.SetErr(errors.New("delete error"))
				s.mockRedis.EXPECT().HDel(ctx, key, fields[0], fields[1]).Return(intCmd).Once()
			})

			s.ExecuteScenario(tc, func() error {
				return s.cache.HDelete(ctx, key, fields...)
			})
		})
	}
}

func TestRedisCacheStorageTestSuite(t *testing.T) {
	suite.Run(t, new(RedisCacheStorageTestSuite))
}
