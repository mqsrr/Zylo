//go:build unit

package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	mq "github.com/mqsrr/zylo/social-graph/internal/mq/mocks"
	proto "github.com/mqsrr/zylo/social-graph/internal/protos/github.com/mqsrr/zylo/social-graph/proto/mocks"
	storage "github.com/mqsrr/zylo/social-graph/internal/storage/mocks"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/suite"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

type TestCaseType int

const (
	Success TestCaseType = iota
	NotFound
	CacheHit
	DBFailure
	CacheFailure
	AmqFailure
	S3Failure
)

var mapCaseName = map[TestCaseType]string{
	Success:      "Success",
	NotFound:     "Not Found",
	CacheHit:     "Cache Hit",
	DBFailure:    "DB DBFailure",
	CacheFailure: "Cache Failure",
	AmqFailure:   "Amq DBFailure",
	S3Failure:    "S3 Failure",
}

type UserRelationshipsHandlerTestSuit struct {
	suite.Suite
	mockCache          *storage.MockCacheStorage
	mockStorage        *storage.MockRelationshipStorage
	mockConsumer       *mq.MockConsumer
	mockProfileService *proto.MockUserProfileServiceClient
	server             *Server

	testTypeHandlers map[TestCaseType]func()
}

type TestConfig struct {
	caseType     TestCaseType
	httpMethod   string
	requestUri   string
	expectedCode int
}

func (s *UserRelationshipsHandlerTestSuit) SetupTest() {
	t := s.T()
	s.mockStorage = storage.NewMockRelationshipStorage(t)
	s.mockCache = storage.NewMockCacheStorage(t)
	s.mockConsumer = mq.NewMockConsumer(t)
	s.mockProfileService = proto.NewMockUserProfileServiceClient(t)
	s.testTypeHandlers = make(map[TestCaseType]func())

	s.server = NewServer(&config.Config{
		PublicConfig: nil,
		DB:           nil,
		Redis:        nil,
		Jwt: &config.JwtConfig{
			Secret:   "SOMELONGKEYTHATISNOTPRESENT333",
			Issuer:   "zylo-testing",
			Audience: "zylo-testing",
		},
		Amqp: nil,
		Grpc: &config.GrpcClientConfig{
			ServerAddr: "https://mocked",
		},
	}, s.mockStorage, s.mockCache, s.mockConsumer, s.mockProfileService)

	s.mockConsumer.EXPECT().Consume(mock.Anything, mock.Anything).Return(nil)

	err := s.server.MountHandlers()
	s.Require().NoError(err, "Could not setup handlers")
}

func (s *UserRelationshipsHandlerTestSuit) generateTestToken() string {
	claims := map[string]interface{}{
		"user_id": ulid.Make(),
		"aud":     s.server.cfg.Jwt.Audience,
		"iss":     s.server.cfg.Jwt.Issuer,
		"exp":     time.Now().Add(time.Hour).Unix(),
	}

	_, tokenString, _ := tokenAuth.Encode(claims)
	return tokenString
}

func (s *UserRelationshipsHandlerTestSuit) prepareRequest(method, path string) (*http.Request, *httptest.ResponseRecorder) {
	req, err := http.NewRequest(method, path, nil)
	rr := httptest.NewRecorder()

	assert.Nil(s.T(), err)
	return req, rr
}

func (s *UserRelationshipsHandlerTestSuit) On(testType TestCaseType, handler func()) *UserRelationshipsHandlerTestSuit {
	s.testTypeHandlers[testType] = handler
	return s
}

func (s *UserRelationshipsHandlerTestSuit) ExecuteScenario(tc TestConfig) (*http.Request, *httptest.ResponseRecorder) {
	s.testTypeHandlers[tc.caseType]()

	req, rr := s.prepareRequest(tc.httpMethod, tc.requestUri)
	req.Header.Add("Authorization", fmt.Sprintf("Bearer %s", s.generateTestToken()))

	s.server.ServeHTTP(rr, req)

	assert.Equal(s.T(), tc.expectedCode, rr.Code)
	return req, rr
}

func TestUserRelationshipsHandlerTestSuit(t *testing.T) {
	s := new(UserRelationshipsHandlerTestSuit)
	suite.Run(t, s)
}

func createMockedUserWithRelationships(userID string) *types.UserWithRelationships {
	return &types.UserWithRelationships{
		User: createMockedUser(userID),
	}
}

func createdFileMetadata() *types.FileMetadata {
	return &types.FileMetadata{
		AccessUrl:   "MockedURl",
		FileName:    "mocked",
		ContentType: "image/jpg",
	}
}

func createMockedUser(userID string) *types.User {
	return &types.User{
		ID:           ulid.MustParse(userID),
		Username:     "test",
		ProfileImage: createdFileMetadata(),
		Name:         "testName",
		CreatedAt:    time.Now().String(),
	}

}

func createArrayOfMockedUsers(userID string) []*types.User {
	users := make([]*types.User, 5)
	for i := 0; i < 5; i++ {
		users[i] = createMockedUser(userID)
	}

	return users
}

func (s *UserRelationshipsHandlerTestSuit) setupS3ImageServiceForUsers(users []*types.User, err error) {
	fileMetadata := createdFileMetadata()
	if err != nil {
		s.mockS3Service.EXPECT().GetPresignedUrl(mock.Anything, mock.Anything).Return(fileMetadata, err).Once()
		return
	}

	for i := range users {
		s.mockS3Service.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", users[i].ID)).Return(fileMetadata, err).Once()
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleGetUserWithRelationships() {
	t := s.T()
	userID := ulid.Make().String()
	expectedUser := createMockedUserWithRelationships(userID)

	fileMetadata := createdFileMetadata()
	expectedResponse, _ := json.Marshal(expectedUser)

	failureErr := errors.New("database error")
	cacheFailureErr := errors.New("cache error")
	s3FailureErr := errors.New("profileServiceClient error")

	uri := fmt.Sprintf("/api/users/%s/relationships", userID)
	tests := []TestConfig{
		{Success, "GET", uri, http.StatusOK},
		{CacheHit, "GET", uri, http.StatusOK},
		{DBFailure, "GET", uri, http.StatusInternalServerError},
		{S3Failure, "GET", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", userID, mock.AnythingOfType("*types.UserWithRelationships")).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetUserWithRelationships(mock.Anything, userID).Return(expectedUser, nil).Once()

					s.mockS3Service.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", expectedUser.User.ID)).Return(fileMetadata, nil).Once()
					s.mockCache.EXPECT().HSet(mock.Anything, "SocialGraph", userID, expectedUser, time.Duration(s.server.cfg.S3.PresignedUrlExpire)*time.Minute).Return(nil).Once()
				}).
				On(CacheHit, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", userID, mock.AnythingOfType("*types.UserWithRelationships")).Return(nil).Once()
				}).
				On(DBFailure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", userID, mock.AnythingOfType("*types.UserWithRelationships")).Return(failureErr).Once()
					s.mockStorage.EXPECT().GetUserWithRelationships(mock.Anything, userID).Return(expectedUser, failureErr).Once()
				}).
				On(S3Failure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", userID, mock.AnythingOfType("*types.UserWithRelationships")).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetUserWithRelationships(mock.Anything, userID).Return(expectedUser, nil).Once()
					s.mockS3Service.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", expectedUser.User.ID)).Return(fileMetadata, s3FailureErr).Once()
				})

			_, rr := s.ExecuteScenario(tc)
			if tc.caseType == Success {
				assert.JSONEq(t, fmt.Sprintf("%s", expectedResponse), rr.Body.String())
			}
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleGetFollowers() {
	t := s.T()
	userID := ulid.Make().String()
	expectedUsers := createArrayOfMockedUsers(userID)
	expectedResponse, _ := json.Marshal(expectedUsers)

	failureErr := errors.New("database error")
	cacheFailureErr := errors.New("cache error")
	s3FailureErr := errors.New("profileServiceClient error")

	uri := fmt.Sprintf("/api/users/%s/followers", userID)
	tests := []TestConfig{
		{Success, "GET", uri, http.StatusOK},
		{CacheHit, "GET", uri, http.StatusOK},
		{DBFailure, "GET", uri, http.StatusInternalServerError},
		{S3Failure, "GET", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowersKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFollowers(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, nil)
					s.mockCache.EXPECT().HSet(mock.Anything, "SocialGraph", GetFollowersKey(userID), expectedUsers, time.Minute).Return(nil)
				}).
				On(CacheHit, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowersKey(userID), mock.Anything).Return(nil).Once()
				}).
				On(DBFailure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowersKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFollowers(mock.Anything, userID).Return(expectedUsers, failureErr).Once()
				}).
				On(S3Failure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowersKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFollowers(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, s3FailureErr)
				})

			_, rr := s.ExecuteScenario(tc)
			if tc.caseType == Success {
				assert.JSONEq(t, fmt.Sprintf("%s", expectedResponse), rr.Body.String())
			}
		})
	}
}
func (s *UserRelationshipsHandlerTestSuit) TestHandleGetFollowed() {
	t := s.T()
	userID := ulid.Make().String()
	expectedUsers := createArrayOfMockedUsers(userID)
	expectedResponse, _ := json.Marshal(expectedUsers)

	failureErr := errors.New("database error")
	cacheFailureErr := errors.New("cache error")
	s3FailureErr := errors.New("profileServiceClient error")

	uri := fmt.Sprintf("/api/users/%s/followers/me", userID)
	tests := []TestConfig{
		{Success, "GET", uri, http.StatusOK},
		{CacheHit, "GET", uri, http.StatusOK},
		{DBFailure, "GET", uri, http.StatusInternalServerError},
		{S3Failure, "GET", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowedKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFollowedPeople(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, nil)
					s.mockCache.EXPECT().HSet(mock.Anything, "SocialGraph", GetFollowedKey(userID), expectedUsers, time.Minute).Return(nil).Once()
				}).
				On(CacheHit, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowedKey(userID), mock.Anything).Return(nil).Once()
				}).
				On(DBFailure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowedKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFollowedPeople(mock.Anything, userID).Return(expectedUsers, failureErr).Once()
				}).
				On(S3Failure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFollowedKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFollowedPeople(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, s3FailureErr)
				})

			_, rr := s.ExecuteScenario(tc)
			if tc.caseType == Success {
				assert.JSONEq(t, fmt.Sprintf("%s", expectedResponse), rr.Body.String())
			}
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleGetBlocked() {
	t := s.T()
	userID := ulid.Make().String()
	expectedUsers := createArrayOfMockedUsers(userID)
	expectedResponse, _ := json.Marshal(expectedUsers)

	failureErr := errors.New("database error")
	cacheFailureErr := errors.New("cache error")
	s3FailureErr := errors.New("profileServiceClient error")

	uri := fmt.Sprintf("/api/users/%s/blocks", userID)
	tests := []TestConfig{
		{Success, "GET", uri, http.StatusOK},
		{CacheHit, "GET", uri, http.StatusOK},
		{DBFailure, "GET", uri, http.StatusInternalServerError},
		{S3Failure, "GET", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetBlockedKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetBlockedPeople(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, nil)
					s.mockCache.EXPECT().HSet(mock.Anything, "SocialGraph", GetBlockedKey(userID), expectedUsers, time.Minute).Return(nil).Once()
				}).
				On(CacheHit, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetBlockedKey(userID), mock.Anything).Return(nil).Once()
				}).
				On(DBFailure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetBlockedKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetBlockedPeople(mock.Anything, userID).Return(expectedUsers, failureErr).Once()
				}).
				On(S3Failure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetBlockedKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetBlockedPeople(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, s3FailureErr)
				})

			_, rr := s.ExecuteScenario(tc)
			if tc.caseType == Success {
				assert.JSONEq(t, fmt.Sprintf("%s", expectedResponse), rr.Body.String())
			}
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleGetFriends() {
	t := s.T()
	userID := ulid.Make().String()
	expectedUsers := createArrayOfMockedUsers(userID)
	expectedResponse, _ := json.Marshal(expectedUsers)

	failureErr := errors.New("database error")
	cacheFailureErr := errors.New("cache error")
	s3FailureErr := errors.New("profileServiceClient error")

	uri := fmt.Sprintf("/api/users/%s/friends", userID)
	tests := []TestConfig{
		{Success, "GET", uri, http.StatusOK},
		{CacheHit, "GET", uri, http.StatusOK},
		{DBFailure, "GET", uri, http.StatusInternalServerError},
		{S3Failure, "GET", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFriendsKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFriends(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, nil)
					s.mockCache.EXPECT().HSet(mock.Anything, "SocialGraph", GetFriendsKey(userID), expectedUsers, time.Minute).Return(nil).Once()
				}).
				On(CacheHit, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFriendsKey(userID), mock.Anything).Return(nil).Once()
				}).
				On(DBFailure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFriendsKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFriends(mock.Anything, userID).Return(expectedUsers, failureErr).Once()
				}).
				On(S3Failure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetFriendsKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetFriends(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, s3FailureErr)
				})

			_, rr := s.ExecuteScenario(tc)
			if tc.caseType == Success {
				assert.JSONEq(t, fmt.Sprintf("%s", expectedResponse), rr.Body.String())
			}
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleRemoveFriend() {
	t := s.T()
	userID := ulid.Make().String()
	friendID := ulid.Make().String()

	failureErr := errors.New("database error")
	s3FailureErr := errors.New("profileServiceClient error")

	uri := fmt.Sprintf("/api/users/%s/friends/%s", userID, friendID)
	tests := []TestConfig{
		{Success, "DELETE", uri, http.StatusNoContent},
		{NotFound, "DELETE", uri, http.StatusNotFound},
		{DBFailure, "DELETE", uri, http.StatusInternalServerError},
		{S3Failure, "DELETE", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().RemoveFriend(mock.Anything, userID, friendID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFriendsKey(userID), friendID, GetFriendsKey(friendID)).Return(nil).Once()
					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.friends.remove", mock.AnythingOfType("types.UserRemovedFriend")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().RemoveFriend(mock.Anything, userID, friendID).Return(false, nil).Once()
				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().RemoveFriend(mock.Anything, userID, friendID).Return(false, failureErr).Once()
				}).
				On(S3Failure, func() {
					s.mockStorage.EXPECT().RemoveFriend(mock.Anything, userID, friendID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFriendsKey(userID), friendID, GetFriendsKey(friendID)).Return(nil).Once()
					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.friends.remove", mock.AnythingOfType("types.UserRemovedFriend")).Return(s3FailureErr).Once()
				})

			s.ExecuteScenario(tc)
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleGetPendingRequests() {
	t := s.T()
	userID := ulid.Make().String()
	expectedUsers := createArrayOfMockedUsers(userID)
	expectedResponse, _ := json.Marshal(expectedUsers)

	failureErr := errors.New("database error")
	cacheFailureErr := errors.New("cache error")
	s3FailureErr := errors.New("profileServiceClient error")

	uri := fmt.Sprintf("/api/users/%s/friends/requests", userID)
	tests := []TestConfig{
		{Success, "GET", uri, http.StatusOK},
		{CacheHit, "GET", uri, http.StatusOK},
		{DBFailure, "GET", uri, http.StatusInternalServerError},
		{S3Failure, "GET", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetPendingRequestKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetPendingFriendRequests(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, nil)
					s.mockCache.EXPECT().HSet(mock.Anything, "SocialGraph", GetPendingRequestKey(userID), expectedUsers, time.Minute).Return(nil).Once()
				}).
				On(CacheHit, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetPendingRequestKey(userID), mock.Anything).Return(nil).Once()
				}).
				On(DBFailure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetPendingRequestKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetPendingFriendRequests(mock.Anything, userID).Return(expectedUsers, failureErr).Once()
				}).
				On(S3Failure, func() {
					s.mockCache.EXPECT().HGet(mock.Anything, "SocialGraph", GetPendingRequestKey(userID), mock.Anything).Return(cacheFailureErr).Once()
					s.mockStorage.EXPECT().GetPendingFriendRequests(mock.Anything, userID).Return(expectedUsers, nil).Once()

					s.setupS3ImageServiceForUsers(expectedUsers, s3FailureErr)
				})

			_, rr := s.ExecuteScenario(tc)
			if tc.caseType == Success {
				assert.JSONEq(t, fmt.Sprintf("%s", expectedResponse), rr.Body.String())
			}
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleFollowUser() {
	t := s.T()
	userID := ulid.Make().String()
	followedID := ulid.Make().String()

	failureErr := errors.New("database error")
	amqFailureErr := errors.New("amq error")

	uri := fmt.Sprintf("/api/users/%s/followers/%s", userID, followedID)
	tests := []TestConfig{
		{Success, "POST", uri, http.StatusNoContent},
		{NotFound, "POST", uri, http.StatusNotFound},
		{DBFailure, "POST", uri, http.StatusInternalServerError},
		{AmqFailure, "POST", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().FollowUser(mock.Anything, userID, followedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFollowedKey(userID), followedID, GetFollowersKey(followedID)).Return(nil).Once()
					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.followed", mock.AnythingOfType("types.UserFollowedMessage")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().FollowUser(mock.Anything, userID, followedID).Return(false, nil).Once()

				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().FollowUser(mock.Anything, userID, followedID).Return(false, failureErr).Once()

				}).
				On(AmqFailure, func() {
					s.mockStorage.EXPECT().FollowUser(mock.Anything, userID, followedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFollowedKey(userID), followedID, GetFollowersKey(followedID)).Return(nil).Once()
					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.followed", mock.Anything).Return(amqFailureErr).Once()
				})

			s.ExecuteScenario(tc)
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleUnfollowUser() {
	t := s.T()
	userID := ulid.Make().String()
	followedID := ulid.Make().String()

	failureErr := errors.New("database error")
	amqFailureErr := errors.New("amq error")

	uri := fmt.Sprintf("/api/users/%s/followers/%s", userID, followedID)
	tests := []TestConfig{
		{Success, "DELETE", uri, http.StatusNoContent},
		{NotFound, "DELETE", uri, http.StatusNotFound},
		{DBFailure, "DELETE", uri, http.StatusInternalServerError},
		{AmqFailure, "DELETE", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().UnfollowUser(mock.Anything, userID, followedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFollowedKey(userID), followedID, GetFollowersKey(followedID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.unfollowed", mock.AnythingOfType("types.UserUnfollowedMessage")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().UnfollowUser(mock.Anything, userID, followedID).Return(false, nil).Once()
				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().UnfollowUser(mock.Anything, userID, followedID).Return(false, failureErr).Once()
				}).
				On(AmqFailure, func() {
					s.mockStorage.EXPECT().UnfollowUser(mock.Anything, userID, followedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFollowedKey(userID), followedID, GetFollowersKey(followedID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.unfollowed", mock.Anything).Return(amqFailureErr).Once()
				})

			s.ExecuteScenario(tc)
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleSendFriendRequest() {
	t := s.T()
	userID := ulid.Make().String()
	receiverID := ulid.Make().String()

	failureErr := errors.New("database error")
	amqFailureErr := errors.New("amq error")

	uri := fmt.Sprintf("/api/users/%s/friends/requests/%s", userID, receiverID)
	tests := []TestConfig{
		{Success, "POST", uri, http.StatusNoContent},
		{NotFound, "POST", uri, http.StatusNotFound},
		{DBFailure, "POST", uri, http.StatusInternalServerError},
		{AmqFailure, "POST", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().SendFriendRequest(mock.Anything, userID, receiverID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, receiverID, GetPendingRequestKey(userID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.sent.friend", mock.AnythingOfType("types.UserSentFriendRequestMessage")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().SendFriendRequest(mock.Anything, userID, receiverID).Return(false, nil).Once()
				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().SendFriendRequest(mock.Anything, userID, receiverID).Return(false, failureErr).Once()
				}).
				On(AmqFailure, func() {
					s.mockStorage.EXPECT().SendFriendRequest(mock.Anything, userID, receiverID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, receiverID, GetPendingRequestKey(userID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.sent.friend", mock.Anything).Return(amqFailureErr).Once()

				})

			s.ExecuteScenario(tc)
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleAcceptFriendRequest() {
	t := s.T()
	userID := ulid.Make().String()
	receiverID := ulid.Make().String()

	failureErr := errors.New("database error")
	amqFailureErr := errors.New("amq error")

	uri := fmt.Sprintf("/api/users/%s/friends/requests/%s", userID, receiverID)
	tests := []TestConfig{
		{Success, "PUT", uri, http.StatusNoContent},
		{NotFound, "PUT", uri, http.StatusNotFound},
		{DBFailure, "PUT", uri, http.StatusInternalServerError},
		{AmqFailure, "PUT", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().AcceptFriendRequest(mock.Anything, userID, receiverID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFriendsKey(userID), receiverID, GetPendingRequestKey(receiverID), GetFriendsKey(receiverID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.add.friend", mock.AnythingOfType("types.UserAcceptedFriendRequestMessage")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().AcceptFriendRequest(mock.Anything, userID, receiverID).Return(false, nil).Once()
				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().AcceptFriendRequest(mock.Anything, userID, receiverID).Return(false, failureErr).Once()
				}).
				On(AmqFailure, func() {
					s.mockStorage.EXPECT().AcceptFriendRequest(mock.Anything, userID, receiverID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, GetFriendsKey(userID), receiverID, GetPendingRequestKey(receiverID), GetFriendsKey(receiverID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.add.friend", mock.AnythingOfType("types.UserAcceptedFriendRequestMessage")).Return(amqFailureErr).Once()
				})

			s.ExecuteScenario(tc)
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleDeclineFriendRequest() {
	t := s.T()
	userID := ulid.Make().String()
	receiverID := ulid.Make().String()

	failureErr := errors.New("database error")
	amqFailureErr := errors.New("amq error")

	uri := fmt.Sprintf("/api/users/%s/friends/requests/%s", userID, receiverID)
	tests := []TestConfig{
		{Success, "DELETE", uri, http.StatusNoContent},
		{NotFound, "DELETE", uri, http.StatusNotFound},
		{DBFailure, "DELETE", uri, http.StatusInternalServerError},
		{AmqFailure, "DELETE", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().DeclineFriendRequest(mock.Anything, userID, receiverID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, receiverID, GetPendingRequestKey(receiverID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.remove.friend", mock.AnythingOfType("types.UserDeclinedFriendRequestMessage")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().DeclineFriendRequest(mock.Anything, userID, receiverID).Return(false, nil).Once()
				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().DeclineFriendRequest(mock.Anything, userID, receiverID).Return(false, failureErr).Once()
				}).
				On(AmqFailure, func() {
					s.mockStorage.EXPECT().DeclineFriendRequest(mock.Anything, userID, receiverID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDelete(mock.Anything, "SocialGraph", userID, receiverID, GetPendingRequestKey(receiverID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.remove.friend", mock.Anything).Return(amqFailureErr).Once()

				})

			s.ExecuteScenario(tc)
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleBlockUser() {
	t := s.T()
	userID := ulid.Make().String()
	blockedID := ulid.Make().String()

	failureErr := errors.New("database error")
	amqFailureErr := errors.New("amq error")

	uri := fmt.Sprintf("/api/users/%s/blocks/%s", userID, blockedID)
	tests := []TestConfig{
		{Success, "POST", uri, http.StatusNoContent},
		{NotFound, "POST", uri, http.StatusNotFound},
		{DBFailure, "POST", uri, http.StatusInternalServerError},
		{AmqFailure, "POST", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().BlockUser(mock.Anything, userID, blockedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", userID)).Return(nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", blockedID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.blocked", mock.AnythingOfType("types.UserBlockedMessage")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().BlockUser(mock.Anything, userID, blockedID).Return(false, nil).Once()
				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().BlockUser(mock.Anything, userID, blockedID).Return(false, failureErr).Once()
				}).
				On(AmqFailure, func() {
					s.mockStorage.EXPECT().BlockUser(mock.Anything, userID, blockedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", userID)).Return(nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", blockedID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.blocked", mock.Anything).Return(amqFailureErr).Once()

				})

			s.ExecuteScenario(tc)
		})
	}
}

func (s *UserRelationshipsHandlerTestSuit) TestHandleUnblockUser() {
	t := s.T()
	userID := ulid.Make().String()
	blockedID := ulid.Make().String()

	failureErr := errors.New("database error")
	amqFailureErr := errors.New("amq error")

	uri := fmt.Sprintf("/api/users/%s/blocks/%s", userID, blockedID)
	tests := []TestConfig{
		{Success, "DELETE", uri, http.StatusNoContent},
		{NotFound, "DELETE", uri, http.StatusNotFound},
		{DBFailure, "DELETE", uri, http.StatusInternalServerError},
		{AmqFailure, "DELETE", uri, http.StatusInternalServerError},
	}

	for _, tc := range tests {
		t.Run(mapCaseName[tc.caseType], func(t *testing.T) {
			s.
				On(Success, func() {
					s.mockStorage.EXPECT().UnblockUser(mock.Anything, userID, blockedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", userID)).Return(nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", blockedID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.unblocked", mock.AnythingOfType("types.UserUnblockedMessage")).Return(nil).Once()
				}).
				On(NotFound, func() {
					s.mockStorage.EXPECT().UnblockUser(mock.Anything, userID, blockedID).Return(false, nil).Once()
				}).
				On(DBFailure, func() {
					s.mockStorage.EXPECT().UnblockUser(mock.Anything, userID, blockedID).Return(false, failureErr).Once()
				}).
				On(AmqFailure, func() {
					s.mockStorage.EXPECT().UnblockUser(mock.Anything, userID, blockedID).Return(true, nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", userID)).Return(nil).Once()
					s.mockCache.EXPECT().HDeleteAll(mock.Anything, "SocialGraph", fmt.Sprintf("*%s*", blockedID)).Return(nil).Once()

					s.mockConsumer.EXPECT().PublishMessage("user-exchange", "user.unblocked", mock.Anything).Return(amqFailureErr).Once()

				})

			s.ExecuteScenario(tc)
		})
	}
}
