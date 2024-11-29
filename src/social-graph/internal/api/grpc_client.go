package api

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/config"
	"github.com/mqsrr/zylo/social-graph/internal/protos/github.com/mqsrr/zylo/social-graph/proto"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"strings"
	"time"
)

type ProfileService interface {
	GetProfilePicture(ctx context.Context, userId ulid.ULID) (*types.FileMetadata, error)
	CloseConnection() error
}

type UserProfileService struct {
	grpcClient proto.UserProfileServiceClient
	grpcConn   *grpc.ClientConn
}

func (s *UserProfileService) GetProfilePicture(ctx context.Context, userId ulid.ULID) (*types.FileMetadata, error) {
	response, err := s.grpcClient.GetProfilePicture(ctx, &proto.UserProfileRequest{UserId: userId.String()})
	if err != nil {
		return nil, err
	}

	return &types.FileMetadata{
		AccessUrl:   &types.PresignedUrl{Url: response.ProfilePictureUrl, ExpiresIn: time.UnixMilli(response.ExpiresIn)},
		FileName:    response.FileName,
		ContentType: response.ContentType,
	}, nil
}

func (s *UserProfileService) CloseConnection() error {
	return s.grpcConn.Close()
}

func NewProfileService(cfg *config.GrpcClientConfig) (ProfileService, error) {
	grpcServer, _ := strings.CutPrefix(cfg.ServerAddr, "http://")
	conn, err := grpc.NewClient(grpcServer, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, err
	}

	client := proto.NewUserProfileServiceClient(conn)
	return &UserProfileService{
		grpcClient: client,
		grpcConn:   conn,
	}, nil
}
