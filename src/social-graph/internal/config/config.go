package config

import (
	"context"
	"fmt"
	"github.com/Azure/azure-sdk-for-go/sdk/azidentity"
	"github.com/Azure/azure-sdk-for-go/sdk/keyvault/azsecrets"
	"github.com/rs/zerolog/log"
	"os"
	"strconv"
	"time"
)

type SecretClient interface {
	GetSecret(ctx context.Context, name string, version string, options *azsecrets.GetSecretOptions) (azsecrets.GetSecretResponse, error)
}

type PublicConfig struct {
	ListeningAddress string
	Environment      string
}

type Config struct {
	*PublicConfig
	DB    *DbConfig
	Redis *RedisConfig
	Jwt   *JwtConfig
	Amqp  *RabbitmqConfig
	Grpc  *GrpcClientConfig
}

type RabbitmqConfig struct {
	AmqpURI  string
	ConnTag  string
	ConnName string
}

type GrpcClientConfig struct {
	ServerAddr string
}

type JwtConfig struct {
	Secret   string
	Issuer   string
	Audience string
}

type RedisConfig struct {
	ConnectionString string
	Expire           time.Duration
}

type DbConfig struct {
	Uri      string
	Username string
	Password string
}

var DefaultConfig *PublicConfig

func CreateKeyVaultClient() (*azsecrets.Client, error) {
	cred, err := azidentity.NewDefaultAzureCredential(nil)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed to obtain Azure credentials")
		os.Exit(1)
	}

	vaultUrl := os.Getenv("AZURE_KEY_VAULT_URL")
	if vaultUrl == "" {
		log.Fatal().Msg("AZURE_KEY_VAULT_URL environment variable must be set")
		os.Exit(1)
	}

	return azsecrets.NewClient(vaultUrl, cred, nil)
}

func getSecretValue(key string, client SecretClient) string {
	ctx := context.Background()

	secretResp, err := client.GetSecret(ctx, key, "", nil)
	if err == nil && secretResp.Value != nil {
		log.Info().Msg(fmt.Sprintf("Secret %s retrieved from Azure Key Vault", key))
		return *secretResp.Value
	}

	log.Warn().Msg(fmt.Sprintf("Falling back to environment variable for %s", key))
	value := getEnvValue(key, "")

	return value
}

func getEnvValue(key string, fallback string) string {
	value := os.Getenv(key)
	if value == "" && fallback == "" {
		log.Panic().Msg(fmt.Sprintf("$%q must be set", key))
	}

	return value
}
func Load(client SecretClient) *Config {
	amqpConfig := &RabbitmqConfig{
		AmqpURI:  getSecretValue("Zylo-RabbitMq--ConnectionString", client),
		ConnName: "social-graph",
	}

	jwtConfig := &JwtConfig{
		Secret:   getSecretValue("Zylo-Jwt--Secret", client),
		Issuer:   getSecretValue("Zylo-Jwt--Issuer", client),
		Audience: getSecretValue("Zylo-Jwt--Audience", client),
	}

	expireInMin, err := strconv.Atoi(getSecretValue("Zylo-Redis--Expire", client))
	if err != nil {
		log.Panic().Err(err).Msg("could not load the global redis expiration time")
		return nil
	}
	redisConfig := &RedisConfig{
		ConnectionString: getSecretValue("Social-Redis--ConnectionString", client),
		Expire:           time.Duration(expireInMin) * time.Minute,
	}

	dbConfig := &DbConfig{
		Uri:      getSecretValue("Social-Neo4j--Uri", client),
		Username: getSecretValue("Social-Neo4j--Username", client),
		Password: getSecretValue("Social-Neo4j--Password", client),
	}

	grpcConfig := &GrpcClientConfig{
		ServerAddr: getSecretValue("UserManagement-Grpc--ServerAddress", client),
	}

	DefaultConfig = &PublicConfig{
		ListeningAddress: getEnvValue("LISTENING_ADDRESS", ":8091"),
		Environment:      getEnvValue("ENVIRONMENT", "Production"),
	}

	config := &Config{
		PublicConfig: DefaultConfig,
		DB:           dbConfig,
		Redis:        redisConfig,
		Jwt:          jwtConfig,
		Amqp:         amqpConfig,
		Grpc:         grpcConfig,
	}
	return config
}
