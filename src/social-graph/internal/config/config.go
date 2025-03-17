package config

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"github.com/Azure/azure-sdk-for-go/sdk/azidentity"
	"github.com/Azure/azure-sdk-for-go/sdk/security/keyvault/azsecrets"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"os"
	"strconv"
	"strings"
	"time"
)

type SecretClient interface {
	GetSecret(ctx context.Context, name string, version string, options *azsecrets.GetSecretOptions) (azsecrets.GetSecretResponse, error)
}

type Server struct {
	Port        string `json:"port"`
	Environment string `json:"environment"`
	ServiceName string `json:"serviceName"`
}

type Config struct {
	*Server       `json:"server"`
	DB            *Db            `json:"db"`
	Redis         *Redis         `json:"redis"`
	Jwt           *Jwt           `json:"jwt"`
	Amqp          *Rabbitmq      `json:"amqp"`
	GrpcServer    *GrpcServer    `json:"grpcServer"`
	OtelCollector *OtelCollector `json:"otelCollector"`
}

type Rabbitmq struct {
	AmqpURI  string `json:"amqpURI"`
	ConnTag  string `json:"connTag"`
	ConnName string `json:"connName"`
}

type Jwt struct {
	Secret   string `json:"secret"`
	Issuer   string `json:"issuer"`
	Audience string `json:"audience"`
}

type Redis struct {
	ConnectionString string        `json:"connectionString"`
	Expire           time.Duration `json:"expire"`
}

type Db struct {
	Uri      string `json:"uri"`
	Username string `json:"username"`
	Password string `json:"password"`
}
type GrpcServer struct {
	Port string `json:"port"`
}

type OtelCollector struct {
	Address string `json:"address"`
}

var DefaultConfig *Server

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
		log.Debug().Msg(fmt.Sprintf("Secret %s retrieved from Azure Key Vault", key))
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

	DefaultConfig = config.Server
	config.Redis.Expire = config.Redis.Expire * time.Minute
	return &config, nil
}

func loadConfigFromSecretStore() (*Config, error) {
	client, err := CreateKeyVaultClient()
	if err != nil {
		return nil, err
	}

	amqpConfig := &Rabbitmq{
		AmqpURI:  getSecretValue("Zylo-RabbitMq--ConnectionString", client),
		ConnName: "social-graph",
	}

	jwtConfig := &Jwt{
		Secret:   getSecretValue("Zylo-Jwt--Secret", client),
		Issuer:   getSecretValue("Zylo-Jwt--Issuer", client),
		Audience: getSecretValue("Zylo-Jwt--Audience", client),
	}

	expireInMin, err := strconv.Atoi(getSecretValue("Zylo-Redis--Expire", client))
	if err != nil {
		return nil, err
	}
	redisConfig := &Redis{
		ConnectionString: getSecretValue("Social-Redis--ConnectionString", client),
		Expire:           time.Duration(expireInMin) * time.Minute,
	}

	dbConfig := &Db{
		Uri:      getSecretValue("Social-Neo4j--Uri", client),
		Username: getSecretValue("Social-Neo4j--Username", client),
		Password: getSecretValue("Social-Neo4j--Password", client),
	}

	DefaultConfig = &Server{
		Port:        getEnvValue("PORT", ":8080"),
		Environment: getEnvValue("ENVIRONMENT", "Development"),
		ServiceName: "social-graph",
	}

	grpcServerConfig := &GrpcServer{
		Port: getSecretValue("Social-GrpcServer--Port", client),
	}
	otelAddress := getSecretValue("Zylo-OTEL--CollectorAddress", client)
	otelAddress = strings.TrimPrefix(otelAddress, "http://")

	otel := &OtelCollector{
		Address: otelAddress,
	}

	config := &Config{
		Server:        DefaultConfig,
		DB:            dbConfig,
		Redis:         redisConfig,
		Jwt:           jwtConfig,
		Amqp:          amqpConfig,
		GrpcServer:    grpcServerConfig,
		OtelCollector: otel,
	}

	DefaultConfig = config.Server
	return config, nil
}

func Load() (*Config, error) {
	if env := getEnvValue("ENVIRONMENT", "Development"); env != "Production" {
		log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stderr})

		return loadConfigFromFile("config/dev.json")
	}

	return loadConfigFromSecretStore()
}

func GetContainerID() (string, error) {
	file, err := os.Open("/proc/self/cgroup")
	if err != nil {
		return "", err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Text()
		parts := strings.Split(line, "/")
		if len(parts) > 2 {
			id := parts[len(parts)-1]
			if len(id) >= 12 {
				return id, nil
			}
		}
	}
	return "", fmt.Errorf("container ID not found")
}
