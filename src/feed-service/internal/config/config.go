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
	"strings"
	"time"
)

type Server struct {
	Environment string `json:"environment"`
	Port        string `json:"port"`
	ServiceName string `json:"serviceName"`
}

type Config struct {
	*Server       `json:"server"`
	DB            *Neo4j         `json:"db"`
	Redis         *Redis         `json:"redis"`
	Amqp          *Rabbitmq      `json:"amqp"`
	GrpcServer    *GrpcServer    `json:"grpcServer"`
	OtelCollector *OtelCollector `json:"otelCollector"`
}

type Rabbitmq struct {
	AmqpURI  string `json:"amqpURI"`
	ConnTag  string `json:"connTag"`
	ConnName string `json:"connName"`
}

type Redis struct {
	ConnectionString string        `json:"connectionString"`
	Expire           time.Duration `json:"expire"`
}

type Neo4j struct {
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
		log.Debug().Msg(fmt.Sprintf("Secret %s retrieved from Azure Key Vault", key))
		return *secretResp.Value
	}

	log.Warn().Msg(fmt.Sprintf("Falling back to environment variable for %s", key))
	value := getEnvValue(key, "")

	return value
}

func getEnvValue(key string, fallback string) string {
	value := os.Getenv(key)
	if value == "" {
		if fallback == "" {
			log.Fatal().
				Msg(fmt.Sprintf("$%q must be set", key))

			os.Exit(1)
		}

		return fallback
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
	client, err := initKeyVaultClient()
	if err != nil {
		return nil, err
	}

	amqpConfig := &Rabbitmq{
		AmqpURI:  getSecretValue("FeedService-RabbitMq--ConnectionString", client),
		ConnName: "feed-service",
	}

	redisConfig := &Redis{
		ConnectionString: getSecretValue("FeedService-Redis--ConnectionString", client),
		Expire:           10 * time.Minute,
	}

	dbConfig := &Neo4j{
		Uri:      getSecretValue("FeedService-Neo4j--Uri", client),
		Username: getSecretValue("FeedService-Neo4j--Username", client),
		Password: getSecretValue("FeedService-Neo4j--Password", client),
	}

	grpcServer := &GrpcServer{
		Port: getSecretValue("FeedService-GrpcServer--Port", client),
	}

	otelAddress := getSecretValue("Zylo-OTEL--CollectorAddress", client)
	otelCollector := &OtelCollector{
		Address: strings.TrimPrefix(otelAddress, "http://"),
	}

	DefaultConfig = &Server{
		Environment: getEnvValue("ENVIRONMENT", "Development"),
		Port:        grpcServer.Port,
		ServiceName: "feed-service",
	}

	config := &Config{
		Server:        DefaultConfig,
		DB:            dbConfig,
		Redis:         redisConfig,
		Amqp:          amqpConfig,
		GrpcServer:    grpcServer,
		OtelCollector: otelCollector,
	}
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
