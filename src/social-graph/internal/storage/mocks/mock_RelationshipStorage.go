// Code generated by mockery v2.46.0. DO NOT EDIT.

package storage

import (
	context "context"

	types "github.com/mqsrr/zylo/social-graph/internal/types"
	mock "github.com/stretchr/testify/mock"
)

// MockRelationshipStorage is an autogenerated mock type for the RelationshipStorage type
type MockRelationshipStorage struct {
	mock.Mock
}

type MockRelationshipStorage_Expecter struct {
	mock *mock.Mock
}

func (_m *MockRelationshipStorage) EXPECT() *MockRelationshipStorage_Expecter {
	return &MockRelationshipStorage_Expecter{mock: &_m.Mock}
}

// AcceptFriendRequest provides a mock function with given fields: ctx, userID, receiverID
func (_m *MockRelationshipStorage) AcceptFriendRequest(ctx context.Context, userID string, receiverID string) (bool, error) {
	ret := _m.Called(ctx, userID, receiverID)

	if len(ret) == 0 {
		panic("no return value specified for AcceptFriendRequest")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, userID, receiverID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, userID, receiverID)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, userID, receiverID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_AcceptFriendRequest_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'AcceptFriendRequest'
type MockRelationshipStorage_AcceptFriendRequest_Call struct {
	*mock.Call
}

// AcceptFriendRequest is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
//   - receiverID string
func (_e *MockRelationshipStorage_Expecter) AcceptFriendRequest(ctx interface{}, userID interface{}, receiverID interface{}) *MockRelationshipStorage_AcceptFriendRequest_Call {
	return &MockRelationshipStorage_AcceptFriendRequest_Call{Call: _e.mock.On("AcceptFriendRequest", ctx, userID, receiverID)}
}

func (_c *MockRelationshipStorage_AcceptFriendRequest_Call) Run(run func(ctx context.Context, userID string, receiverID string)) *MockRelationshipStorage_AcceptFriendRequest_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_AcceptFriendRequest_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_AcceptFriendRequest_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_AcceptFriendRequest_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_AcceptFriendRequest_Call {
	_c.Call.Return(run)
	return _c
}

// BlockUser provides a mock function with given fields: ctx, blockerID, blockedID
func (_m *MockRelationshipStorage) BlockUser(ctx context.Context, blockerID string, blockedID string) (bool, error) {
	ret := _m.Called(ctx, blockerID, blockedID)

	if len(ret) == 0 {
		panic("no return value specified for BlockUser")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, blockerID, blockedID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, blockerID, blockedID)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, blockerID, blockedID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_BlockUser_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'BlockUser'
type MockRelationshipStorage_BlockUser_Call struct {
	*mock.Call
}

// BlockUser is a helper method to define mock.On call
//   - ctx context.Context
//   - blockerID string
//   - blockedID string
func (_e *MockRelationshipStorage_Expecter) BlockUser(ctx interface{}, blockerID interface{}, blockedID interface{}) *MockRelationshipStorage_BlockUser_Call {
	return &MockRelationshipStorage_BlockUser_Call{Call: _e.mock.On("BlockUser", ctx, blockerID, blockedID)}
}

func (_c *MockRelationshipStorage_BlockUser_Call) Run(run func(ctx context.Context, blockerID string, blockedID string)) *MockRelationshipStorage_BlockUser_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_BlockUser_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_BlockUser_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_BlockUser_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_BlockUser_Call {
	_c.Call.Return(run)
	return _c
}

// CreateUser provides a mock function with given fields: ctx, user
func (_m *MockRelationshipStorage) CreateUser(ctx context.Context, user *types.User) (bool, error) {
	ret := _m.Called(ctx, user)

	if len(ret) == 0 {
		panic("no return value specified for CreateUser")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, *types.User) (bool, error)); ok {
		return rf(ctx, user)
	}
	if rf, ok := ret.Get(0).(func(context.Context, *types.User) bool); ok {
		r0 = rf(ctx, user)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, *types.User) error); ok {
		r1 = rf(ctx, user)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_CreateUser_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'CreateUser'
type MockRelationshipStorage_CreateUser_Call struct {
	*mock.Call
}

// CreateUser is a helper method to define mock.On call
//   - ctx context.Context
//   - user *types.User
func (_e *MockRelationshipStorage_Expecter) CreateUser(ctx interface{}, user interface{}) *MockRelationshipStorage_CreateUser_Call {
	return &MockRelationshipStorage_CreateUser_Call{Call: _e.mock.On("CreateUser", ctx, user)}
}

func (_c *MockRelationshipStorage_CreateUser_Call) Run(run func(ctx context.Context, user *types.User)) *MockRelationshipStorage_CreateUser_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(*types.User))
	})
	return _c
}

func (_c *MockRelationshipStorage_CreateUser_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_CreateUser_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_CreateUser_Call) RunAndReturn(run func(context.Context, *types.User) (bool, error)) *MockRelationshipStorage_CreateUser_Call {
	_c.Call.Return(run)
	return _c
}

// DeclineFriendRequest provides a mock function with given fields: ctx, userID, receiverID
func (_m *MockRelationshipStorage) DeclineFriendRequest(ctx context.Context, userID string, receiverID string) (bool, error) {
	ret := _m.Called(ctx, userID, receiverID)

	if len(ret) == 0 {
		panic("no return value specified for DeclineFriendRequest")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, userID, receiverID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, userID, receiverID)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, userID, receiverID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_DeclineFriendRequest_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'DeclineFriendRequest'
type MockRelationshipStorage_DeclineFriendRequest_Call struct {
	*mock.Call
}

// DeclineFriendRequest is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
//   - receiverID string
func (_e *MockRelationshipStorage_Expecter) DeclineFriendRequest(ctx interface{}, userID interface{}, receiverID interface{}) *MockRelationshipStorage_DeclineFriendRequest_Call {
	return &MockRelationshipStorage_DeclineFriendRequest_Call{Call: _e.mock.On("DeclineFriendRequest", ctx, userID, receiverID)}
}

func (_c *MockRelationshipStorage_DeclineFriendRequest_Call) Run(run func(ctx context.Context, userID string, receiverID string)) *MockRelationshipStorage_DeclineFriendRequest_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_DeclineFriendRequest_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_DeclineFriendRequest_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_DeclineFriendRequest_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_DeclineFriendRequest_Call {
	_c.Call.Return(run)
	return _c
}

// DeleteUserByID provides a mock function with given fields: ctx, userId
func (_m *MockRelationshipStorage) DeleteUserByID(ctx context.Context, userId string) (bool, error) {
	ret := _m.Called(ctx, userId)

	if len(ret) == 0 {
		panic("no return value specified for DeleteUserByID")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string) (bool, error)); ok {
		return rf(ctx, userId)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string) bool); ok {
		r0 = rf(ctx, userId)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string) error); ok {
		r1 = rf(ctx, userId)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_DeleteUserByID_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'DeleteUserByID'
type MockRelationshipStorage_DeleteUserByID_Call struct {
	*mock.Call
}

// DeleteUserByID is a helper method to define mock.On call
//   - ctx context.Context
//   - userId string
func (_e *MockRelationshipStorage_Expecter) DeleteUserByID(ctx interface{}, userId interface{}) *MockRelationshipStorage_DeleteUserByID_Call {
	return &MockRelationshipStorage_DeleteUserByID_Call{Call: _e.mock.On("DeleteUserByID", ctx, userId)}
}

func (_c *MockRelationshipStorage_DeleteUserByID_Call) Run(run func(ctx context.Context, userId string)) *MockRelationshipStorage_DeleteUserByID_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_DeleteUserByID_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_DeleteUserByID_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_DeleteUserByID_Call) RunAndReturn(run func(context.Context, string) (bool, error)) *MockRelationshipStorage_DeleteUserByID_Call {
	_c.Call.Return(run)
	return _c
}

// FollowUser provides a mock function with given fields: ctx, followerId, followedId
func (_m *MockRelationshipStorage) FollowUser(ctx context.Context, followerId string, followedId string) (bool, error) {
	ret := _m.Called(ctx, followerId, followedId)

	if len(ret) == 0 {
		panic("no return value specified for FollowUser")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, followerId, followedId)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, followerId, followedId)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, followerId, followedId)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_FollowUser_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'FollowUser'
type MockRelationshipStorage_FollowUser_Call struct {
	*mock.Call
}

// FollowUser is a helper method to define mock.On call
//   - ctx context.Context
//   - followerId string
//   - followedId string
func (_e *MockRelationshipStorage_Expecter) FollowUser(ctx interface{}, followerId interface{}, followedId interface{}) *MockRelationshipStorage_FollowUser_Call {
	return &MockRelationshipStorage_FollowUser_Call{Call: _e.mock.On("FollowUser", ctx, followerId, followedId)}
}

func (_c *MockRelationshipStorage_FollowUser_Call) Run(run func(ctx context.Context, followerId string, followedId string)) *MockRelationshipStorage_FollowUser_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_FollowUser_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_FollowUser_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_FollowUser_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_FollowUser_Call {
	_c.Call.Return(run)
	return _c
}

// GetBlockedPeople provides a mock function with given fields: ctx, userID
func (_m *MockRelationshipStorage) GetBlockedPeople(ctx context.Context, userID string) ([]*types.User, error) {
	ret := _m.Called(ctx, userID)

	if len(ret) == 0 {
		panic("no return value specified for GetBlockedPeople")
	}

	var r0 []*types.User
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string) ([]*types.User, error)); ok {
		return rf(ctx, userID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string) []*types.User); ok {
		r0 = rf(ctx, userID)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).([]*types.User)
		}
	}

	if rf, ok := ret.Get(1).(func(context.Context, string) error); ok {
		r1 = rf(ctx, userID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_GetBlockedPeople_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'GetBlockedPeople'
type MockRelationshipStorage_GetBlockedPeople_Call struct {
	*mock.Call
}

// GetBlockedPeople is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
func (_e *MockRelationshipStorage_Expecter) GetBlockedPeople(ctx interface{}, userID interface{}) *MockRelationshipStorage_GetBlockedPeople_Call {
	return &MockRelationshipStorage_GetBlockedPeople_Call{Call: _e.mock.On("GetBlockedPeople", ctx, userID)}
}

func (_c *MockRelationshipStorage_GetBlockedPeople_Call) Run(run func(ctx context.Context, userID string)) *MockRelationshipStorage_GetBlockedPeople_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_GetBlockedPeople_Call) Return(_a0 []*types.User, _a1 error) *MockRelationshipStorage_GetBlockedPeople_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_GetBlockedPeople_Call) RunAndReturn(run func(context.Context, string) ([]*types.User, error)) *MockRelationshipStorage_GetBlockedPeople_Call {
	_c.Call.Return(run)
	return _c
}

// GetFollowedPeople provides a mock function with given fields: ctx, userID
func (_m *MockRelationshipStorage) GetFollowedPeople(ctx context.Context, userID string) ([]*types.User, error) {
	ret := _m.Called(ctx, userID)

	if len(ret) == 0 {
		panic("no return value specified for GetFollowedPeople")
	}

	var r0 []*types.User
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string) ([]*types.User, error)); ok {
		return rf(ctx, userID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string) []*types.User); ok {
		r0 = rf(ctx, userID)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).([]*types.User)
		}
	}

	if rf, ok := ret.Get(1).(func(context.Context, string) error); ok {
		r1 = rf(ctx, userID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_GetFollowedPeople_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'GetFollowedPeople'
type MockRelationshipStorage_GetFollowedPeople_Call struct {
	*mock.Call
}

// GetFollowedPeople is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
func (_e *MockRelationshipStorage_Expecter) GetFollowedPeople(ctx interface{}, userID interface{}) *MockRelationshipStorage_GetFollowedPeople_Call {
	return &MockRelationshipStorage_GetFollowedPeople_Call{Call: _e.mock.On("GetFollowedPeople", ctx, userID)}
}

func (_c *MockRelationshipStorage_GetFollowedPeople_Call) Run(run func(ctx context.Context, userID string)) *MockRelationshipStorage_GetFollowedPeople_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_GetFollowedPeople_Call) Return(_a0 []*types.User, _a1 error) *MockRelationshipStorage_GetFollowedPeople_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_GetFollowedPeople_Call) RunAndReturn(run func(context.Context, string) ([]*types.User, error)) *MockRelationshipStorage_GetFollowedPeople_Call {
	_c.Call.Return(run)
	return _c
}

// GetFollowers provides a mock function with given fields: ctx, userID
func (_m *MockRelationshipStorage) GetFollowers(ctx context.Context, userID string) ([]*types.User, error) {
	ret := _m.Called(ctx, userID)

	if len(ret) == 0 {
		panic("no return value specified for GetFollowers")
	}

	var r0 []*types.User
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string) ([]*types.User, error)); ok {
		return rf(ctx, userID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string) []*types.User); ok {
		r0 = rf(ctx, userID)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).([]*types.User)
		}
	}

	if rf, ok := ret.Get(1).(func(context.Context, string) error); ok {
		r1 = rf(ctx, userID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_GetFollowers_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'GetFollowers'
type MockRelationshipStorage_GetFollowers_Call struct {
	*mock.Call
}

// GetFollowers is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
func (_e *MockRelationshipStorage_Expecter) GetFollowers(ctx interface{}, userID interface{}) *MockRelationshipStorage_GetFollowers_Call {
	return &MockRelationshipStorage_GetFollowers_Call{Call: _e.mock.On("GetFollowers", ctx, userID)}
}

func (_c *MockRelationshipStorage_GetFollowers_Call) Run(run func(ctx context.Context, userID string)) *MockRelationshipStorage_GetFollowers_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_GetFollowers_Call) Return(_a0 []*types.User, _a1 error) *MockRelationshipStorage_GetFollowers_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_GetFollowers_Call) RunAndReturn(run func(context.Context, string) ([]*types.User, error)) *MockRelationshipStorage_GetFollowers_Call {
	_c.Call.Return(run)
	return _c
}

// GetFriends provides a mock function with given fields: ctx, userID
func (_m *MockRelationshipStorage) GetFriends(ctx context.Context, userID string) ([]*types.User, error) {
	ret := _m.Called(ctx, userID)

	if len(ret) == 0 {
		panic("no return value specified for GetFriends")
	}

	var r0 []*types.User
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string) ([]*types.User, error)); ok {
		return rf(ctx, userID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string) []*types.User); ok {
		r0 = rf(ctx, userID)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).([]*types.User)
		}
	}

	if rf, ok := ret.Get(1).(func(context.Context, string) error); ok {
		r1 = rf(ctx, userID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_GetFriends_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'GetFriends'
type MockRelationshipStorage_GetFriends_Call struct {
	*mock.Call
}

// GetFriends is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
func (_e *MockRelationshipStorage_Expecter) GetFriends(ctx interface{}, userID interface{}) *MockRelationshipStorage_GetFriends_Call {
	return &MockRelationshipStorage_GetFriends_Call{Call: _e.mock.On("GetFriends", ctx, userID)}
}

func (_c *MockRelationshipStorage_GetFriends_Call) Run(run func(ctx context.Context, userID string)) *MockRelationshipStorage_GetFriends_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_GetFriends_Call) Return(_a0 []*types.User, _a1 error) *MockRelationshipStorage_GetFriends_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_GetFriends_Call) RunAndReturn(run func(context.Context, string) ([]*types.User, error)) *MockRelationshipStorage_GetFriends_Call {
	_c.Call.Return(run)
	return _c
}

// GetPendingFriendRequests provides a mock function with given fields: ctx, userID
func (_m *MockRelationshipStorage) GetPendingFriendRequests(ctx context.Context, userID string) ([]*types.User, error) {
	ret := _m.Called(ctx, userID)

	if len(ret) == 0 {
		panic("no return value specified for GetPendingFriendRequests")
	}

	var r0 []*types.User
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string) ([]*types.User, error)); ok {
		return rf(ctx, userID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string) []*types.User); ok {
		r0 = rf(ctx, userID)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).([]*types.User)
		}
	}

	if rf, ok := ret.Get(1).(func(context.Context, string) error); ok {
		r1 = rf(ctx, userID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_GetPendingFriendRequests_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'GetPendingFriendRequests'
type MockRelationshipStorage_GetPendingFriendRequests_Call struct {
	*mock.Call
}

// GetPendingFriendRequests is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
func (_e *MockRelationshipStorage_Expecter) GetPendingFriendRequests(ctx interface{}, userID interface{}) *MockRelationshipStorage_GetPendingFriendRequests_Call {
	return &MockRelationshipStorage_GetPendingFriendRequests_Call{Call: _e.mock.On("GetPendingFriendRequests", ctx, userID)}
}

func (_c *MockRelationshipStorage_GetPendingFriendRequests_Call) Run(run func(ctx context.Context, userID string)) *MockRelationshipStorage_GetPendingFriendRequests_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_GetPendingFriendRequests_Call) Return(_a0 []*types.User, _a1 error) *MockRelationshipStorage_GetPendingFriendRequests_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_GetPendingFriendRequests_Call) RunAndReturn(run func(context.Context, string) ([]*types.User, error)) *MockRelationshipStorage_GetPendingFriendRequests_Call {
	_c.Call.Return(run)
	return _c
}

// GetUserWithRelationships provides a mock function with given fields: ctx, userID
func (_m *MockRelationshipStorage) GetUserWithRelationships(ctx context.Context, userID string) (*types.UserWithRelationships, error) {
	ret := _m.Called(ctx, userID)

	if len(ret) == 0 {
		panic("no return value specified for GetUserWithRelationships")
	}

	var r0 *types.UserWithRelationships
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string) (*types.UserWithRelationships, error)); ok {
		return rf(ctx, userID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string) *types.UserWithRelationships); ok {
		r0 = rf(ctx, userID)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).(*types.UserWithRelationships)
		}
	}

	if rf, ok := ret.Get(1).(func(context.Context, string) error); ok {
		r1 = rf(ctx, userID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_GetUserWithRelationships_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'GetUserWithRelationships'
type MockRelationshipStorage_GetUserWithRelationships_Call struct {
	*mock.Call
}

// GetUserWithRelationships is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
func (_e *MockRelationshipStorage_Expecter) GetUserWithRelationships(ctx interface{}, userID interface{}) *MockRelationshipStorage_GetUserWithRelationships_Call {
	return &MockRelationshipStorage_GetUserWithRelationships_Call{Call: _e.mock.On("GetUserWithRelationships", ctx, userID)}
}

func (_c *MockRelationshipStorage_GetUserWithRelationships_Call) Run(run func(ctx context.Context, userID string)) *MockRelationshipStorage_GetUserWithRelationships_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_GetUserWithRelationships_Call) Return(_a0 *types.UserWithRelationships, _a1 error) *MockRelationshipStorage_GetUserWithRelationships_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_GetUserWithRelationships_Call) RunAndReturn(run func(context.Context, string) (*types.UserWithRelationships, error)) *MockRelationshipStorage_GetUserWithRelationships_Call {
	_c.Call.Return(run)
	return _c
}

// RemoveFriend provides a mock function with given fields: ctx, userID, friendID
func (_m *MockRelationshipStorage) RemoveFriend(ctx context.Context, userID string, friendID string) (bool, error) {
	ret := _m.Called(ctx, userID, friendID)

	if len(ret) == 0 {
		panic("no return value specified for RemoveFriend")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, userID, friendID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, userID, friendID)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, userID, friendID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_RemoveFriend_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'RemoveFriend'
type MockRelationshipStorage_RemoveFriend_Call struct {
	*mock.Call
}

// RemoveFriend is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
//   - friendID string
func (_e *MockRelationshipStorage_Expecter) RemoveFriend(ctx interface{}, userID interface{}, friendID interface{}) *MockRelationshipStorage_RemoveFriend_Call {
	return &MockRelationshipStorage_RemoveFriend_Call{Call: _e.mock.On("RemoveFriend", ctx, userID, friendID)}
}

func (_c *MockRelationshipStorage_RemoveFriend_Call) Run(run func(ctx context.Context, userID string, friendID string)) *MockRelationshipStorage_RemoveFriend_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_RemoveFriend_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_RemoveFriend_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_RemoveFriend_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_RemoveFriend_Call {
	_c.Call.Return(run)
	return _c
}

// SendFriendRequest provides a mock function with given fields: ctx, userID, receiverID
func (_m *MockRelationshipStorage) SendFriendRequest(ctx context.Context, userID string, receiverID string) (bool, error) {
	ret := _m.Called(ctx, userID, receiverID)

	if len(ret) == 0 {
		panic("no return value specified for SendFriendRequest")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, userID, receiverID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, userID, receiverID)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, userID, receiverID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_SendFriendRequest_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'SendFriendRequest'
type MockRelationshipStorage_SendFriendRequest_Call struct {
	*mock.Call
}

// SendFriendRequest is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
//   - receiverID string
func (_e *MockRelationshipStorage_Expecter) SendFriendRequest(ctx interface{}, userID interface{}, receiverID interface{}) *MockRelationshipStorage_SendFriendRequest_Call {
	return &MockRelationshipStorage_SendFriendRequest_Call{Call: _e.mock.On("SendFriendRequest", ctx, userID, receiverID)}
}

func (_c *MockRelationshipStorage_SendFriendRequest_Call) Run(run func(ctx context.Context, userID string, receiverID string)) *MockRelationshipStorage_SendFriendRequest_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_SendFriendRequest_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_SendFriendRequest_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_SendFriendRequest_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_SendFriendRequest_Call {
	_c.Call.Return(run)
	return _c
}

// UnblockUser provides a mock function with given fields: ctx, blockerID, blockedID
func (_m *MockRelationshipStorage) UnblockUser(ctx context.Context, blockerID string, blockedID string) (bool, error) {
	ret := _m.Called(ctx, blockerID, blockedID)

	if len(ret) == 0 {
		panic("no return value specified for UnblockUser")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, blockerID, blockedID)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, blockerID, blockedID)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, blockerID, blockedID)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_UnblockUser_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'UnblockUser'
type MockRelationshipStorage_UnblockUser_Call struct {
	*mock.Call
}

// UnblockUser is a helper method to define mock.On call
//   - ctx context.Context
//   - blockerID string
//   - blockedID string
func (_e *MockRelationshipStorage_Expecter) UnblockUser(ctx interface{}, blockerID interface{}, blockedID interface{}) *MockRelationshipStorage_UnblockUser_Call {
	return &MockRelationshipStorage_UnblockUser_Call{Call: _e.mock.On("UnblockUser", ctx, blockerID, blockedID)}
}

func (_c *MockRelationshipStorage_UnblockUser_Call) Run(run func(ctx context.Context, blockerID string, blockedID string)) *MockRelationshipStorage_UnblockUser_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_UnblockUser_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_UnblockUser_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_UnblockUser_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_UnblockUser_Call {
	_c.Call.Return(run)
	return _c
}

// UnfollowUser provides a mock function with given fields: ctx, followerId, followedId
func (_m *MockRelationshipStorage) UnfollowUser(ctx context.Context, followerId string, followedId string) (bool, error) {
	ret := _m.Called(ctx, followerId, followedId)

	if len(ret) == 0 {
		panic("no return value specified for UnfollowUser")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string) (bool, error)); ok {
		return rf(ctx, followerId, followedId)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string) bool); ok {
		r0 = rf(ctx, followerId, followedId)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string) error); ok {
		r1 = rf(ctx, followerId, followedId)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_UnfollowUser_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'UnfollowUser'
type MockRelationshipStorage_UnfollowUser_Call struct {
	*mock.Call
}

// UnfollowUser is a helper method to define mock.On call
//   - ctx context.Context
//   - followerId string
//   - followedId string
func (_e *MockRelationshipStorage_Expecter) UnfollowUser(ctx interface{}, followerId interface{}, followedId interface{}) *MockRelationshipStorage_UnfollowUser_Call {
	return &MockRelationshipStorage_UnfollowUser_Call{Call: _e.mock.On("UnfollowUser", ctx, followerId, followedId)}
}

func (_c *MockRelationshipStorage_UnfollowUser_Call) Run(run func(ctx context.Context, followerId string, followedId string)) *MockRelationshipStorage_UnfollowUser_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_UnfollowUser_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_UnfollowUser_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_UnfollowUser_Call) RunAndReturn(run func(context.Context, string, string) (bool, error)) *MockRelationshipStorage_UnfollowUser_Call {
	_c.Call.Return(run)
	return _c
}

// UpdateUser provides a mock function with given fields: ctx, userID, name, bio, location
func (_m *MockRelationshipStorage) UpdateUser(ctx context.Context, userID string, name string, bio string, location string) (bool, error) {
	ret := _m.Called(ctx, userID, name, bio, location)

	if len(ret) == 0 {
		panic("no return value specified for UpdateUser")
	}

	var r0 bool
	var r1 error
	if rf, ok := ret.Get(0).(func(context.Context, string, string, string, string) (bool, error)); ok {
		return rf(ctx, userID, name, bio, location)
	}
	if rf, ok := ret.Get(0).(func(context.Context, string, string, string, string) bool); ok {
		r0 = rf(ctx, userID, name, bio, location)
	} else {
		r0 = ret.Get(0).(bool)
	}

	if rf, ok := ret.Get(1).(func(context.Context, string, string, string, string) error); ok {
		r1 = rf(ctx, userID, name, bio, location)
	} else {
		r1 = ret.Error(1)
	}

	return r0, r1
}

// MockRelationshipStorage_UpdateUser_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'UpdateUser'
type MockRelationshipStorage_UpdateUser_Call struct {
	*mock.Call
}

// UpdateUser is a helper method to define mock.On call
//   - ctx context.Context
//   - userID string
//   - name string
//   - bio string
//   - location string
func (_e *MockRelationshipStorage_Expecter) UpdateUser(ctx interface{}, userID interface{}, name interface{}, bio interface{}, location interface{}) *MockRelationshipStorage_UpdateUser_Call {
	return &MockRelationshipStorage_UpdateUser_Call{Call: _e.mock.On("UpdateUser", ctx, userID, name, bio, location)}
}

func (_c *MockRelationshipStorage_UpdateUser_Call) Run(run func(ctx context.Context, userID string, name string, bio string, location string)) *MockRelationshipStorage_UpdateUser_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string), args[3].(string), args[4].(string))
	})
	return _c
}

func (_c *MockRelationshipStorage_UpdateUser_Call) Return(_a0 bool, _a1 error) *MockRelationshipStorage_UpdateUser_Call {
	_c.Call.Return(_a0, _a1)
	return _c
}

func (_c *MockRelationshipStorage_UpdateUser_Call) RunAndReturn(run func(context.Context, string, string, string, string) (bool, error)) *MockRelationshipStorage_UpdateUser_Call {
	_c.Call.Return(run)
	return _c
}

// NewMockRelationshipStorage creates a new instance of MockRelationshipStorage. It also registers a testing interface on the mock and a cleanup function to assert the mocks expectations.
// The first argument is typically a *testing.T value.
func NewMockRelationshipStorage(t interface {
	mock.TestingT
	Cleanup(func())
}) *MockRelationshipStorage {
	mock := &MockRelationshipStorage{}
	mock.Mock.Test(t)

	t.Cleanup(func() { mock.AssertExpectations(t) })

	return mock
}
