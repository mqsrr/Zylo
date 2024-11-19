package config

import (
	"context"
	"fmt"
	"github.com/Azure/azure-sdk-for-go/sdk/azidentity"
	"github.com/Azure/azure-sdk-for-go/sdk/security/keyvault/azsecrets"
	"github.com/joho/godotenv"
	"github.com/rs/zerolog/log"
	"os"
	"strconv"
	"time"
)

type PublicConfig struct {
	ListeningAddress string
	Environment      string
}

type Config struct {
	*PublicConfig
	DB    *Neo4jConfig
	Redis *RedisConfig
	Jwt   *JwtConfig
	Amqp  *RabbitmqConfig
	S3    *S3Config
}

type RabbitmqConfig struct {
	AmqpURI  string
	ConnTag  string
	ConnName string
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

type Neo4jConfig struct {
	Uri      string
	Username string
	Password string
}

type S3Config struct {
	BucketName         string
	PresignedUrlExpire int
}

var DefaultConfig *PublicConfig

func initKeyVaultClient() (*azsecrets.Client, error) {
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

func getSecretValue(key string, client *azsecrets.Client) string {
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
		log.Fatal().
			Msg(fmt.Sprintf("$%q must be set", key))

		os.Exit(1)
	}

	return fallback
}
func Load() *Config {
	if err := godotenv.Load(); err != nil {
		log.Warn().Err(err).Msg("Error loading .config file, falling back to environment variables")
	}

	client, err := initKeyVaultClient()
	if err != nil {
		return nil
	}

	amqpConfig := &RabbitmqConfig{
		AmqpURI:  getSecretValue("FeedService-RabbitMq--ConnectionString", client),
		ConnName: "feed-service",
	}

	jwtConfig := &JwtConfig{
		Secret:   getSecretValue("Zylo-Jwt--Secret", client),
		Issuer:   getSecretValue("Zylo-Jwt--Issuer", client),
		Audience: getSecretValue("Zylo-Jwt--Audience", client),
	}

	redisConfig := &RedisConfig{
		ConnectionString: getSecretValue("FeedService-Redis--ConnectionString", client),
		Expire:           10 * time.Minute,
	}

	dbConfig := &Neo4jConfig{
		Uri:      getSecretValue("FeedService-Neo4j--Uri", client),
		Username: getSecretValue("FeedService-Neo4j--Username", client),
		Password: getSecretValue("FeedService-Neo4j--Password", client),
	}

	value, err := strconv.Atoi(getSecretValue("Zylo-S3--PresignedUrlExpire", client))
	if err != nil {
		return nil
	}

	s3Config := &S3Config{
		BucketName:         getSecretValue("Zylo-S3--BucketName", client),
		PresignedUrlExpire: value,
	}

	DefaultConfig = &PublicConfig{
		ListeningAddress: getEnvValue("LISTENING_ADDRESS", ":8092"),
		Environment:      getEnvValue("ENVIRONMENT", "Development"),
	}

	config := &Config{
		PublicConfig: DefaultConfig,
		DB:           dbConfig,
		Redis:        redisConfig,
		Jwt:          jwtConfig,
		Amqp:         amqpConfig,
		S3:           s3Config,
	}
	return config
}
