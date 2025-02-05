package api

import (
	"context"
	"github.com/mqsrr/zylo/social-graph/internal/protos/github.com/mqsrr/zylo/social-graph/proto"
	"github.com/mqsrr/zylo/social-graph/internal/storage"
	"github.com/mqsrr/zylo/social-graph/internal/types"
	"github.com/oklog/ulid/v2"
	"google.golang.org/protobuf/types/known/timestamppb"
	"time"
)

type RelationshipServiceServer struct {
	proto.UnimplementedRelationshipServiceServer
	storage storage.RelationshipStorage
}

func NewRelationshipServiceServer(s storage.RelationshipStorage) *RelationshipServiceServer {
	return &RelationshipServiceServer{
		storage: s,
	}
}

func (s *RelationshipServiceServer) GetUserRelationships(ctx context.Context, req *proto.RelationshipRequest) (*proto.RelationshipResponse, error) {
	userID, err := ulid.Parse(req.GetUserId())
	if err != nil {
		return nil, types.GrpcError(err)
	}

	uwr, err := s.storage.GetUserWithRelationships(ctx, userID)
	if err != nil {
		return nil, types.GrpcError(err)
	}

	resp, err := convertUserWithRelationshipsToProto(uwr)
	if err != nil {
		return nil, types.GrpcError(err)
	}

	return resp, nil
}

func (s *RelationshipServiceServer) GetBatchRelationships(ctx context.Context, req *proto.BatchRelationshipRequest) (*proto.BatchRelationshipResponse, error) {
	ids, err := convertStringsToULIDs(req.GetUserIds())
	if err != nil {
		return nil, types.GrpcError(err)
	}
	users, err := s.storage.BatchGetUserWithRelationships(ctx, ids)
	if err != nil {
		return nil, types.GrpcError(err)
	}

	resp, err := convertUserMap(users)
	if err != nil {
		return nil, types.GrpcError(err)
	}

	return &proto.BatchRelationshipResponse{Users: resp}, nil
}

func convertRelationship(rel *types.Relationship) *proto.RelationshipData {
	if rel == nil {
		return &proto.RelationshipData{
			Ids:       []string{},
			CreatedAt: map[string]*timestamppb.Timestamp{},
		}
	}

	createdAtMap := make(map[string]*timestamppb.Timestamp, len(rel.CreatedAt))
	for id, tsStr := range rel.CreatedAt {
		t, err := time.Parse(time.RFC3339, tsStr)
		if err != nil {
			continue
		}
		createdAtMap[id.String()] = timestamppb.New(t)
	}

	ids := make([]string, len(rel.IDs))
	for i, id := range rel.IDs {
		ids[i] = id.String()
	}

	return &proto.RelationshipData{
		Ids:       ids,
		CreatedAt: createdAtMap,
	}
}

func convertUserWithRelationshipsToProto(uwr *types.UserWithRelationships) (*proto.RelationshipResponse, error) {
	if uwr == nil {
		return nil, types.NewBadRequest("User is null")
	}

	var friendReqs *proto.FriendRequests
	if uwr.FriendRequests != nil {
		friendReqs = &proto.FriendRequests{
			Sent:     convertRelationship(uwr.FriendRequests.Sent),
			Received: convertRelationship(uwr.FriendRequests.Received),
		}
	} else {
		friendReqs = &proto.FriendRequests{}
	}

	var follows *proto.FollowRequest
	if uwr.Follows != nil {
		follows = &proto.FollowRequest{
			Followers: convertRelationship(uwr.Follows.Followers),
			Following: convertRelationship(uwr.Follows.Following),
		}
	} else {
		follows = &proto.FollowRequest{}
	}

	return &proto.RelationshipResponse{
		UserId: uwr.UserID.String(),
		Relationships: &proto.Relationships{
			Friends:        convertRelationship(uwr.Friends),
			Blocks:         convertRelationship(uwr.Blocks),
			FriendRequests: friendReqs,
			Follows:        follows,
		},
	}, nil
}

func convertStringsToULIDs(ids []string) ([]ulid.ULID, error) {
	ulids := make([]ulid.ULID, 0, len(ids))
	for _, idStr := range ids {
		id, err := ulid.Parse(idStr)
		if err != nil {
			return nil, types.NewBadRequestErr("Invalid user id", err)
		}
		ulids = append(ulids, id)
	}
	return ulids, nil
}

func convertUserMap(in map[ulid.ULID]*types.UserWithRelationships) (map[string]*proto.RelationshipResponse, error) {
	out := make(map[string]*proto.RelationshipResponse, len(in))
	for uid, uwr := range in {
		protoResp, err := convertUserWithRelationshipsToProto(uwr)
		if err != nil {
			return nil, err
		}
		out[uid.String()] = protoResp
	}
	return out, nil
}
