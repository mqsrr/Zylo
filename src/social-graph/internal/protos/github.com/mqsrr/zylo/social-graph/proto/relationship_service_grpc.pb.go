// Code generated by protoc-gen-go-grpc. DO NOT EDIT.
// versions:
// - protoc-gen-go-grpc v1.5.1
// - protoc             v5.29.1
// source: relationship_service.proto

package proto

import (
	context "context"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
// Requires gRPC-Go v1.64.0 or later.
const _ = grpc.SupportPackageIsVersion9

const (
	RelationshipService_GetUserRelationships_FullMethodName  = "/relationship_service.RelationshipService/GetUserRelationships"
	RelationshipService_GetBatchRelationships_FullMethodName = "/relationship_service.RelationshipService/GetBatchRelationships"
)

// RelationshipServiceClient is the client API for RelationshipService service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type RelationshipServiceClient interface {
	GetUserRelationships(ctx context.Context, in *RelationshipRequest, opts ...grpc.CallOption) (*RelationshipResponse, error)
	GetBatchRelationships(ctx context.Context, in *BatchRelationshipRequest, opts ...grpc.CallOption) (*BatchRelationshipResponse, error)
}

type relationshipServiceClient struct {
	cc grpc.ClientConnInterface
}

func NewRelationshipServiceClient(cc grpc.ClientConnInterface) RelationshipServiceClient {
	return &relationshipServiceClient{cc}
}

func (c *relationshipServiceClient) GetUserRelationships(ctx context.Context, in *RelationshipRequest, opts ...grpc.CallOption) (*RelationshipResponse, error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	out := new(RelationshipResponse)
	err := c.cc.Invoke(ctx, RelationshipService_GetUserRelationships_FullMethodName, in, out, cOpts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *relationshipServiceClient) GetBatchRelationships(ctx context.Context, in *BatchRelationshipRequest, opts ...grpc.CallOption) (*BatchRelationshipResponse, error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	out := new(BatchRelationshipResponse)
	err := c.cc.Invoke(ctx, RelationshipService_GetBatchRelationships_FullMethodName, in, out, cOpts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// RelationshipServiceServer is the server API for RelationshipService service.
// All implementations must embed UnimplementedRelationshipServiceServer
// for forward compatibility.
type RelationshipServiceServer interface {
	GetUserRelationships(context.Context, *RelationshipRequest) (*RelationshipResponse, error)
	GetBatchRelationships(context.Context, *BatchRelationshipRequest) (*BatchRelationshipResponse, error)
	mustEmbedUnimplementedRelationshipServiceServer()
}

// UnimplementedRelationshipServiceServer must be embedded to have
// forward compatible implementations.
//
// NOTE: this should be embedded by value instead of pointer to avoid a nil
// pointer dereference when methods are called.
type UnimplementedRelationshipServiceServer struct{}

func (UnimplementedRelationshipServiceServer) GetUserRelationships(context.Context, *RelationshipRequest) (*RelationshipResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GetUserRelationships not implemented")
}
func (UnimplementedRelationshipServiceServer) GetBatchRelationships(context.Context, *BatchRelationshipRequest) (*BatchRelationshipResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GetBatchRelationships not implemented")
}
func (UnimplementedRelationshipServiceServer) mustEmbedUnimplementedRelationshipServiceServer() {}
func (UnimplementedRelationshipServiceServer) testEmbeddedByValue()                             {}

// UnsafeRelationshipServiceServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to RelationshipServiceServer will
// result in compilation errors.
type UnsafeRelationshipServiceServer interface {
	mustEmbedUnimplementedRelationshipServiceServer()
}

func RegisterRelationshipServiceServer(s grpc.ServiceRegistrar, srv RelationshipServiceServer) {
	// If the following call pancis, it indicates UnimplementedRelationshipServiceServer was
	// embedded by pointer and is nil.  This will cause panics if an
	// unimplemented method is ever invoked, so we test this at initialization
	// time to prevent it from happening at runtime later due to I/O.
	if t, ok := srv.(interface{ testEmbeddedByValue() }); ok {
		t.testEmbeddedByValue()
	}
	s.RegisterService(&RelationshipService_ServiceDesc, srv)
}

func _RelationshipService_GetUserRelationships_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(RelationshipRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(RelationshipServiceServer).GetUserRelationships(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: RelationshipService_GetUserRelationships_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(RelationshipServiceServer).GetUserRelationships(ctx, req.(*RelationshipRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _RelationshipService_GetBatchRelationships_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(BatchRelationshipRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(RelationshipServiceServer).GetBatchRelationships(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: RelationshipService_GetBatchRelationships_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(RelationshipServiceServer).GetBatchRelationships(ctx, req.(*BatchRelationshipRequest))
	}
	return interceptor(ctx, in, info, handler)
}

// RelationshipService_ServiceDesc is the grpc.ServiceDesc for RelationshipService service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var RelationshipService_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "relationship_service.RelationshipService",
	HandlerType: (*RelationshipServiceServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "GetUserRelationships",
			Handler:    _RelationshipService_GetUserRelationships_Handler,
		},
		{
			MethodName: "GetBatchRelationships",
			Handler:    _RelationshipService_GetBatchRelationships_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "relationship_service.proto",
}
