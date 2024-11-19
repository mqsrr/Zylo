//go:build unit

package config

import (
	"errors"
	"github.com/Azure/azure-sdk-for-go/sdk/keyvault/azsecrets"
	config "github.com/mqsrr/zylo/social-graph/internal/config/mocks"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/suite"
	"os"
	"testing"
	"time"
)

type ConfigTestSuite struct {
	suite.Suite
	mockClient *config.MockSecretClient
}

func (s *ConfigTestSuite) SetupTest() {
	s.mockClient = config.NewMockSecretClient(s.T())

	err := os.Setenv("LISTENING_ADDRESS", ":8091")
	s.Require().NoError(err)

	err = os.Setenv("AZURE_KEY_VAULT_URL", "https:/localhost:test")
	s.Require().NoError(err)

	err = os.Setenv("ENVIRONMENT", "Production")
	s.Require().NoError(err)
}

func (s *ConfigTestSuite) TearDownTest() {
	os.Clearenv()
}

func (s *ConfigTestSuite) TestLoadConfigSuccess() {

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-RabbitMq--ConnectionString", "", mock.Anything).
		Return(createAzureSecret("amqp://guest:guest@localhost:5672/"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-Jwt--Secret", "", mock.Anything).
		Return(createAzureSecret("supersecretkey"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-Jwt--Issuer", "", mock.Anything).
		Return(createAzureSecret("zylo-issuer"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-Jwt--Audience", "", mock.Anything).
		Return(createAzureSecret("zylo-audience"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Redis--ConnectionString", "", mock.Anything).
		Return(createAzureSecret("redis://localhost:6379/0"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Neo4j--Uri", "", mock.Anything).
		Return(createAzureSecret("bolt://localhost:7687"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Neo4j--Username", "", mock.Anything).
		Return(createAzureSecret("neo4j"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Neo4j--Password", "", mock.Anything).
		Return(createAzureSecret("password"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-S3--BucketName", "", mock.Anything).
		Return(createAzureSecret("bucketname"), nil)

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-S3--PresignedUrlExpire", "", mock.Anything).
		Return(createAzureSecret("60"), nil)

	// Load config
	cfg := Load(s.mockClient)

	// Assertions
	assert.NotNil(s.T(), cfg)
	assert.Equal(s.T(), ":8091", cfg.ListeningAddress)
	assert.Equal(s.T(), "Production", cfg.Environment)
	assert.Equal(s.T(), "amqp://guest:guest@localhost:5672/", cfg.Amqp.AmqpURI)
	assert.Equal(s.T(), "supersecretkey", cfg.Jwt.Secret)
	assert.Equal(s.T(), "zylo-issuer", cfg.Jwt.Issuer)
	assert.Equal(s.T(), "zylo-audience", cfg.Jwt.Audience)
	assert.Equal(s.T(), "redis://localhost:6379/0", cfg.Redis.ConnectionString)
	assert.Equal(s.T(), 10*time.Minute, cfg.Redis.Expire)
	assert.Equal(s.T(), "bolt://localhost:7687", cfg.DB.Uri)
	assert.Equal(s.T(), "neo4j", cfg.DB.Username)
	assert.Equal(s.T(), "password", cfg.DB.Password)
}

func (s *ConfigTestSuite) TestLoadConfigWithFallback() {

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-RabbitMq--ConnectionString", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-Jwt--Secret", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-Jwt--Issuer", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-Jwt--Audience", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Redis--ConnectionString", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Neo4j--Uri", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Neo4j--Username", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Social-Neo4j--Password", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-S3--BucketName", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-S3--PresignedUrlExpire", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-Jwt--Secret", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	_ = os.Setenv("Zylo-RabbitMq--ConnectionString", "fallbacksecret")
	_ = os.Setenv("Zylo-Jwt--Secret", "fallbacksecret")
	_ = os.Setenv("Zylo-Jwt--Issuer", "fallbacksecret")
	_ = os.Setenv("Zylo-Jwt--Audience", "fallbacksecret")
	_ = os.Setenv("Social-Redis--ConnectionString", "fallbacksecret")
	_ = os.Setenv("Social-Neo4j--Uri", "fallbacksecret")
	_ = os.Setenv("Social-Neo4j--Username", "fallbacksecret")
	_ = os.Setenv("Social-Neo4j--Password", "fallbacksecret")
	_ = os.Setenv("Zylo-S3--BucketName", "fallbacksecret")
	_ = os.Setenv("Zylo-S3--PresignedUrlExpire", "fallbacksecret")
	_ = os.Setenv("Zylo-Jwt--Secret", "fallbacksecret")

	cfg := Load(s.mockClient)

	assert.Equal(s.T(), "fallbacksecret", cfg.Jwt.Secret)
}

func (s *ConfigTestSuite) TestLoadConfigFailure() {
	s.mockClient.EXPECT().GetSecret(mock.Anything, "Zylo-RabbitMq--ConnectionString", "", mock.Anything).
		Return(azsecrets.GetSecretResponse{}, errors.New("secret not found"))

	assert.Panics(s.T(), func() {
		Load(s.mockClient)
	})
}

func newString(value string) *string {
	return &value
}

func createAzureSecret(value string) azsecrets.GetSecretResponse {
	return azsecrets.GetSecretResponse{
		SecretBundle: azsecrets.SecretBundle{
			Attributes:  nil,
			ContentType: nil,
			ID:          nil,
			Tags:        nil,
			Value:       newString(value),
			Kid:         nil,
			Managed:     nil,
		},
	}
}

func TestConfigTestSuite(t *testing.T) {
	suite.Run(t, new(ConfigTestSuite))
}
