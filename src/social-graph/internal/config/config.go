package config

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/Azure/azure-sdk-for-go/sdk/azidentity"
	"github.com/Azure/azure-sdk-for-go/sdk/security/keyvault/azsecrets"
	"github.com/rs/zerolog/log"
	"os"
	"strconv"
	"time"
)

type SecretClient interface {
	GetSecret(ctx context.Context, name string, version string, options *azsecrets.GetSecretOptions) (azsecrets.GetSecretResponse, error)
}

type ServerConfig struct {
	Port        string `json:"port"`
	Environment string `json:"environment"`
}

type Config struct {
	*ServerConfig `json:"serverConfig"`
	DB            *DbConfig         `json:"db"`
	Redis         *RedisConfig      `json:"redis"`
	Jwt           *JwtConfig        `json:"jwt"`
	Amqp          *RabbitmqConfig   `json:"amqp"`
	GrpcServer    *GrpcServerConfig `json:"grpcServer"`
}

type RabbitmqConfig struct {
	AmqpURI  string `json:"amqpURI"`
	ConnTag  string `json:"connTag"`
	ConnName string `json:"connName"`
}

type JwtConfig struct {
	Secret   string `json:"secret"`
	Issuer   string `json:"issuer"`
	Audience string `json:"audience"`
}

type RedisConfig struct {
	ConnectionString string        `json:"connectionString"`
	Expire           time.Duration `json:"expire"`
}

type DbConfig struct {
	Uri      string `json:"uri"`
	Username string `json:"username"`
	Password string `json:"password"`
}
type GrpcServerConfig struct {
	Port string `json:"port"`
}

var DefaultConfig *ServerConfig

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
	if value == "" && fallback != "" {
		return fallback
	}

	if value == "" {
		log.Panic().Msg(fmt.Sprintf("$%q must be set", key))
	}

	return value
}

func loadConfigFromFile(filePath string) (*Config, error) {
	f, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer func(f *os.File) {
		err := f.Close()
		if err != nil {
			log.Error().
				Timestamp().
				Caller().
				Str("filePath", filePath).
				Err(err).
				Msg("Failed to close file")
		}
	}(f)

	var config Config
	decoder := json.NewDecoder(f)
	if err := decoder.Decode(&config); err != nil {
		return nil, err
	}

	DefaultConfig = config.ServerConfig
	config.Redis.Expire = config.Redis.Expire * time.Minute
	return &config, nil
}

func loadConfigFromSecretStore() (*Config, error) {
	client, err := CreateKeyVaultClient()
	if err != nil {
		return nil, err
	}

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
		return nil, err
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

	DefaultConfig = &ServerConfig{
		Port:        getEnvValue("PORT", ":8080"),
		Environment: getEnvValue("ENVIRONMENT", "Development"),
	}

	grpcServerConfig := &GrpcServerConfig{
		Port: getSecretValue("Social-GrpcServer--Port", client),
	}

	config := &Config{
		ServerConfig: DefaultConfig,
		DB:           dbConfig,
		Redis:        redisConfig,
		Jwt:          jwtConfig,
		Amqp:         amqpConfig,
		GrpcServer:   grpcServerConfig,
	}

	DefaultConfig = config.ServerConfig
	return config, nil
}

func Load() (*Config, error) {
	if env := getEnvValue("ENVIRONMENT", "Development"); env != "Production" {
		return loadConfigFromFile("config/dev.json")
	}

	return loadConfigFromSecretStore()
}
