//go:build integration

package api_test

import (
	"encoding/json"
	"fmt"
	"github.com/go-chi/jwtauth/v5"
	"github.com/lestrrat-go/jwx/v2/jwt"
	"github.com/mqsrr/zylo/social-graph/internal/testutil"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"github.com/stretchr/testify/mock"
	"io"
	"net/http"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"
)

type APITestSuite struct {
	suite.Suite
	client *http.Client
}

func TestMain(m *testing.M) {
	err := testutil.StartTestContainers()
	if err != nil {
		fmt.Printf("Failed to set up shared containers: %v\n", err)
		os.Exit(1)
	}

	code := m.Run()

	err = testutil.TearDown()
	if err != nil {
		fmt.Printf("Error during container cleanup: %v\n", err)
	}

	os.Exit(code)
}
func (suite *APITestSuite) SetupSuite() {
	testutil.SetupTestServer(suite.T())
	suite.client = &http.Client{}
}

func (suite *APITestSuite) generateTestToken(userID string) string {
	claims := map[string]interface{}{
		"user_id": userID,
		"aud":     testutil.Cfg.Jwt.Audience,
		"iss":     testutil.Cfg.Jwt.Issuer,
		"exp":     time.Now().Add(time.Hour).Unix(),
	}

	tokenAuth := jwtauth.New(
		"HS256",
		[]byte(testutil.Cfg.Jwt.Secret),
		nil,
		jwt.WithAcceptableSkew(5*time.Second),
		jwt.WithAudience(testutil.Cfg.Jwt.Audience),
		jwt.WithIssuer(testutil.Cfg.Jwt.Issuer))

	_, tokenString, _ := tokenAuth.Encode(claims)
	return tokenString
}

func (suite *APITestSuite) createUser(userID ulid.ULID) *types.User {
	user := &types.User{
		ID:        userID,
		Username:  "username",
		Name:      "user",
		CreatedAt: time.Now().UTC().Format(time.RFC3339),
	}

	_, err := testutil.RelationshipStorage.CreateUser(testutil.Ctx, user)
	suite.Require().NoError(err, "Failed to create user")

	return user
}

func (suite *APITestSuite) createFileMetadata() *types.FileMetadata {
	return &types.FileMetadata{
		AccessUrl: &types.PresignedUrl{
			Url:       "mockedUrl",
			ExpiresIn: time.Now(),
		},
		FileName:    "mocked",
		ContentType: "image/jpg",
	}
}

func (suite *APITestSuite) sendRequest(req *http.Request) (*http.Response, []byte) {
	resp, err := suite.client.Do(req)
	suite.Require().NoError(err, "Failed to send request")
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	suite.Require().NoError(err, "Failed to read response body")

	return resp, body
}

func (suite *APITestSuite) makeAuthorizedRequest(method, path string, token string) (*http.Request, error) {
	req, err := http.NewRequest(method, fmt.Sprintf("%s%s", testutil.HttpTestServer.URL, path), nil)
	if err != nil {
		return nil, err
	}
	req.Header.Add("Authorization", "Bearer "+token)
	return req, nil
}

func (suite *APITestSuite) TestHandleGetUserWithRelationships() {
	userID := ulid.Make()
	mockedFile := suite.createFileMetadata()
	suite.createUser(userID)

	testutil.S3ImageServiceMock.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", userID)).Return(mockedFile, nil).Once()

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("GET", fmt.Sprintf("/api/users/%s/relationships", userID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, body := suite.sendRequest(req)
	suite.Equal(http.StatusOK, resp.StatusCode, "Expected status OK")

	var userWithRelationships types.UserWithRelationships
	err = json.Unmarshal(body, &userWithRelationships)
	suite.Require().NoError(err, "Failed to unmarshal response body")

	suite.Equal(userID, userWithRelationships.User.ID, "User ID mismatch")
	suite.Empty(userWithRelationships.Followers, "Expected no followers")
	suite.Empty(userWithRelationships.FollowedPeople, "Expected no followed people")
}

func (suite *APITestSuite) TestHandleGetFollowers() {
	userID := ulid.Make()
	followerID := ulid.Make()
	mockedFile := suite.createFileMetadata()

	suite.createUser(userID)
	suite.createUser(followerID)

	testutil.S3ImageServiceMock.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", followerID)).Return(mockedFile, nil).Once()

	_, err := testutil.RelationshipStorage.FollowUser(testutil.Ctx, followerID.String(), userID.String())
	suite.Require().NoError(err, "Failed to follow user")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("GET", fmt.Sprintf("/api/users/%s/followers", userID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, body := suite.sendRequest(req)
	suite.Equal(http.StatusOK, resp.StatusCode, "Expected status OK")

	var followers []*types.User
	err = json.Unmarshal(body, &followers)
	suite.Require().NoError(err, "Failed to unmarshal response body")

	suite.Equal(1, len(followers), "Expected one follower")
	suite.Equal(followerID, followers[0].ID, "Follower ID mismatch")
}

func (suite *APITestSuite) TestHandleFollow() {
	userID := ulid.Make()
	followerID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(followerID)

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("POST", fmt.Sprintf("/api/users/%s/followers/%s", userID, followerID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "Expected status No Content")
}

func (suite *APITestSuite) TestHandleUnfollow() {
	userID := ulid.Make()
	followerID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(followerID)

	_, err := testutil.RelationshipStorage.FollowUser(testutil.Ctx, userID.String(), followerID.String())
	suite.Require().NoError(err, "Failed to follow user")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("DELETE", fmt.Sprintf("/api/users/%s/followers/%s", userID, followerID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "Expected status No Content")
}

func (suite *APITestSuite) TestHandleGetFollowedPeople() {
	userID := ulid.Make()
	followerID := ulid.Make()
	mockedFile := suite.createFileMetadata()

	suite.createUser(userID)
	suite.createUser(followerID)

	testutil.S3ImageServiceMock.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", userID)).Return(mockedFile, nil).Once()

	_, err := testutil.RelationshipStorage.FollowUser(testutil.Ctx, followerID.String(), userID.String())
	suite.Require().NoError(err, "Failed to follow user")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("GET", fmt.Sprintf("/api/users/%s/followers/me", followerID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, body := suite.sendRequest(req)
	suite.Equal(http.StatusOK, resp.StatusCode, "Expected status OK")

	var followed []*types.User
	err = json.Unmarshal(body, &followed)
	suite.Require().NoError(err, "Failed to unmarshal response body")

	suite.Equal(1, len(followed), "Expected one followed")
	suite.Equal(userID, followed[0].ID, "Follower ID mismatch")
}

func (suite *APITestSuite) TestHandleGetFriends() {
	userID := ulid.Make()
	friendID := ulid.Make()
	mockedFile := suite.createFileMetadata()

	suite.createUser(userID)
	suite.createUser(friendID)

	testutil.S3ImageServiceMock.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", friendID)).Return(mockedFile, nil).Once()

	_, err := testutil.RelationshipStorage.SendFriendRequest(testutil.Ctx, userID.String(), friendID.String())
	suite.Require().NoError(err, "Failed to send request")

	_, err = testutil.RelationshipStorage.AcceptFriendRequest(testutil.Ctx, friendID.String(), userID.String())
	suite.Require().NoError(err, "Failed to accept request")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("GET", fmt.Sprintf("/api/users/%s/friends", userID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, body := suite.sendRequest(req)
	suite.Equal(http.StatusOK, resp.StatusCode, "Expected status OK")

	var friends []*types.User
	err = json.Unmarshal(body, &friends)
	suite.Require().NoError(err, "Failed to unmarshal response body")

	suite.Equal(1, len(friends), "Expected one friend")
	suite.Equal(friendID, friends[0].ID, "Friend ID mismatch")
}

func (suite *APITestSuite) TestHandleRemoveFriend() {
	userID := ulid.Make()
	friendID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(friendID)

	_, err := testutil.RelationshipStorage.SendFriendRequest(testutil.Ctx, userID.String(), friendID.String())
	suite.Require().NoError(err, "Failed to send request")

	_, err = testutil.RelationshipStorage.AcceptFriendRequest(testutil.Ctx, friendID.String(), userID.String())
	suite.Require().NoError(err, "Failed to accept request")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("DELETE", fmt.Sprintf("/api/users/%s/friends/%s", userID, friendID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "User was not deleted")
}

func (suite *APITestSuite) TestHandleGetPendingFriendRequests() {
	userID := ulid.Make()
	friendID := ulid.Make()
	mockedFile := suite.createFileMetadata()

	suite.createUser(userID)
	suite.createUser(friendID)

	testutil.S3ImageServiceMock.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", userID)).Return(mockedFile, nil).Once()

	_, err := testutil.RelationshipStorage.SendFriendRequest(testutil.Ctx, userID.String(), friendID.String())
	suite.Require().NoError(err, "Failed to send request")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("GET", fmt.Sprintf("/api/users/%s/friends/requests", friendID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, body := suite.sendRequest(req)
	suite.Equal(http.StatusOK, resp.StatusCode, "Expected status OK")

	var pendingRequests []*types.User
	err = json.Unmarshal(body, &pendingRequests)
	suite.Require().NoError(err, "Failed to unmarshal response body")

	suite.Equal(1, len(pendingRequests), "Expected one request")
	suite.Equal(userID, pendingRequests[0].ID, "User ID mismatch")
}

func (suite *APITestSuite) TestHandleSendFriendRequest() {
	userID := ulid.Make()
	friendID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(friendID)

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("POST", fmt.Sprintf("/api/users/%s/friends/requests/%s", userID, friendID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "Expected status No Content")
}

func (suite *APITestSuite) TestHandleAcceptFriendRequest() {
	userID := ulid.Make()
	friendID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(friendID)

	_, err := testutil.RelationshipStorage.SendFriendRequest(testutil.Ctx, userID.String(), friendID.String())
	suite.Require().NoError(err, "Failed to send request")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("PUT", fmt.Sprintf("/api/users/%s/friends/requests/%s", friendID, userID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "Expected status No Content")
}

func (suite *APITestSuite) TestHandleDeclineFriendRequest() {
	userID := ulid.Make()
	friendID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(friendID)

	_, err := testutil.RelationshipStorage.SendFriendRequest(testutil.Ctx, userID.String(), friendID.String())
	suite.Require().NoError(err, "Failed to send request")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("DELETE", fmt.Sprintf("/api/users/%s/friends/requests/%s", friendID, userID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "Expected status No Content")
}

func (suite *APITestSuite) TestHandleGetBlockedPeople() {
	userID := ulid.Make()
	blockedID := ulid.Make()
	mockedFile := suite.createFileMetadata()

	suite.createUser(userID)
	suite.createUser(blockedID)

	testutil.S3ImageServiceMock.EXPECT().GetPresignedUrl(mock.Anything, fmt.Sprintf("profile_images/%s", blockedID)).Return(mockedFile, nil).Once()

	_, err := testutil.RelationshipStorage.BlockUser(testutil.Ctx, userID.String(), blockedID.String())
	suite.Require().NoError(err, "Failed to send request")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("GET", fmt.Sprintf("/api/users/%s/blocks", userID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, body := suite.sendRequest(req)
	suite.Equal(http.StatusOK, resp.StatusCode, "Expected status OK")

	var blocked []*types.User
	err = json.Unmarshal(body, &blocked)
	suite.Require().NoError(err, "Failed to unmarshal response body")

	suite.Equal(1, len(blocked), "Expected one blocked person")
	suite.Equal(blockedID, blocked[0].ID, "Follower ID mismatch")
}

func (suite *APITestSuite) TestHandleBlockUser() {
	userID := ulid.Make()
	blockedID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(blockedID)

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("POST", fmt.Sprintf("/api/users/%s/blocks/%s", userID, blockedID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "Expected status No Content")
}

func (suite *APITestSuite) TestHandleUnBlockUser() {
	userID := ulid.Make()
	blockedID := ulid.Make()

	suite.createUser(userID)
	suite.createUser(blockedID)

	_, err := testutil.RelationshipStorage.BlockUser(testutil.Ctx, userID.String(), blockedID.String())
	suite.Require().NoError(err, "Failed to send request")

	tokenString := suite.generateTestToken(userID.String())
	req, err := suite.makeAuthorizedRequest("DELETE", fmt.Sprintf("/api/users/%s/blocks/%s", userID, blockedID), tokenString)
	suite.Require().NoError(err, "Failed to create request")

	resp, _ := suite.sendRequest(req)
	suite.Equal(http.StatusNoContent, resp.StatusCode, "Expected status No Content")
}

func TestAPITestSuite(t *testing.T) {
	apiTestSuite := new(APITestSuite)

	suite.Run(t, apiTestSuite)
}
