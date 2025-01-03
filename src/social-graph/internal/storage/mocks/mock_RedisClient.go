// Code generated by mockery v2.46.0. DO NOT EDIT.

package storage

import (
	context "context"

	redis "github.com/redis/go-redis/v9"
	mock "github.com/stretchr/testify/mock"

	time "time"
)

// MockRedisClient is an autogenerated mock type for the RedisClient type
type MockRedisClient struct {
	mock.Mock
}

type MockRedisClient_Expecter struct {
	mock *mock.Mock
}

func (_m *MockRedisClient) EXPECT() *MockRedisClient_Expecter {
	return &MockRedisClient_Expecter{mock: &_m.Mock}
}

// HDel provides a mock function with given fields: ctx, key, fields
func (_m *MockRedisClient) HDel(ctx context.Context, key string, fields ...string) *redis.IntCmd {
	_va := make([]interface{}, len(fields))
	for _i := range fields {
		_va[_i] = fields[_i]
	}
	var _ca []interface{}
	_ca = append(_ca, ctx, key)
	_ca = append(_ca, _va...)
	ret := _m.Called(_ca...)

	if len(ret) == 0 {
		panic("no return value specified for HDel")
	}

	var r0 *redis.IntCmd
	if rf, ok := ret.Get(0).(func(context.Context, string, ...string) *redis.IntCmd); ok {
		r0 = rf(ctx, key, fields...)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).(*redis.IntCmd)
		}
	}

	return r0
}

// MockRedisClient_HDel_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'HDel'
type MockRedisClient_HDel_Call struct {
	*mock.Call
}

// HDel is a helper method to define mock.On call
//   - ctx context.Context
//   - key string
//   - fields ...string
func (_e *MockRedisClient_Expecter) HDel(ctx interface{}, key interface{}, fields ...interface{}) *MockRedisClient_HDel_Call {
	return &MockRedisClient_HDel_Call{Call: _e.mock.On("HDel",
		append([]interface{}{ctx, key}, fields...)...)}
}

func (_c *MockRedisClient_HDel_Call) Run(run func(ctx context.Context, key string, fields ...string)) *MockRedisClient_HDel_Call {
	_c.Call.Run(func(args mock.Arguments) {
		variadicArgs := make([]string, len(args)-2)
		for i, a := range args[2:] {
			if a != nil {
				variadicArgs[i] = a.(string)
			}
		}
		run(args[0].(context.Context), args[1].(string), variadicArgs...)
	})
	return _c
}

func (_c *MockRedisClient_HDel_Call) Return(_a0 *redis.IntCmd) *MockRedisClient_HDel_Call {
	_c.Call.Return(_a0)
	return _c
}

func (_c *MockRedisClient_HDel_Call) RunAndReturn(run func(context.Context, string, ...string) *redis.IntCmd) *MockRedisClient_HDel_Call {
	_c.Call.Return(run)
	return _c
}

// HExpire provides a mock function with given fields: ctx, key, expiration, fields
func (_m *MockRedisClient) HExpire(ctx context.Context, key string, expiration time.Duration, fields ...string) *redis.IntSliceCmd {
	_va := make([]interface{}, len(fields))
	for _i := range fields {
		_va[_i] = fields[_i]
	}
	var _ca []interface{}
	_ca = append(_ca, ctx, key, expiration)
	_ca = append(_ca, _va...)
	ret := _m.Called(_ca...)

	if len(ret) == 0 {
		panic("no return value specified for HExpire")
	}

	var r0 *redis.IntSliceCmd
	if rf, ok := ret.Get(0).(func(context.Context, string, time.Duration, ...string) *redis.IntSliceCmd); ok {
		r0 = rf(ctx, key, expiration, fields...)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).(*redis.IntSliceCmd)
		}
	}

	return r0
}

// MockRedisClient_HExpire_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'HExpire'
type MockRedisClient_HExpire_Call struct {
	*mock.Call
}

// HExpire is a helper method to define mock.On call
//   - ctx context.Context
//   - key string
//   - expiration time.Duration
//   - fields ...string
func (_e *MockRedisClient_Expecter) HExpire(ctx interface{}, key interface{}, expiration interface{}, fields ...interface{}) *MockRedisClient_HExpire_Call {
	return &MockRedisClient_HExpire_Call{Call: _e.mock.On("HExpire",
		append([]interface{}{ctx, key, expiration}, fields...)...)}
}

func (_c *MockRedisClient_HExpire_Call) Run(run func(ctx context.Context, key string, expiration time.Duration, fields ...string)) *MockRedisClient_HExpire_Call {
	_c.Call.Run(func(args mock.Arguments) {
		variadicArgs := make([]string, len(args)-3)
		for i, a := range args[3:] {
			if a != nil {
				variadicArgs[i] = a.(string)
			}
		}
		run(args[0].(context.Context), args[1].(string), args[2].(time.Duration), variadicArgs...)
	})
	return _c
}

func (_c *MockRedisClient_HExpire_Call) Return(_a0 *redis.IntSliceCmd) *MockRedisClient_HExpire_Call {
	_c.Call.Return(_a0)
	return _c
}

func (_c *MockRedisClient_HExpire_Call) RunAndReturn(run func(context.Context, string, time.Duration, ...string) *redis.IntSliceCmd) *MockRedisClient_HExpire_Call {
	_c.Call.Return(run)
	return _c
}

// HGet provides a mock function with given fields: ctx, key, field
func (_m *MockRedisClient) HGet(ctx context.Context, key string, field string) *redis.StringCmd {
	ret := _m.Called(ctx, key, field)

	if len(ret) == 0 {
		panic("no return value specified for HGet")
	}

	var r0 *redis.StringCmd
	if rf, ok := ret.Get(0).(func(context.Context, string, string) *redis.StringCmd); ok {
		r0 = rf(ctx, key, field)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).(*redis.StringCmd)
		}
	}

	return r0
}

// MockRedisClient_HGet_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'HGet'
type MockRedisClient_HGet_Call struct {
	*mock.Call
}

// HGet is a helper method to define mock.On call
//   - ctx context.Context
//   - key string
//   - field string
func (_e *MockRedisClient_Expecter) HGet(ctx interface{}, key interface{}, field interface{}) *MockRedisClient_HGet_Call {
	return &MockRedisClient_HGet_Call{Call: _e.mock.On("HGet", ctx, key, field)}
}

func (_c *MockRedisClient_HGet_Call) Run(run func(ctx context.Context, key string, field string)) *MockRedisClient_HGet_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(string))
	})
	return _c
}

func (_c *MockRedisClient_HGet_Call) Return(_a0 *redis.StringCmd) *MockRedisClient_HGet_Call {
	_c.Call.Return(_a0)
	return _c
}

func (_c *MockRedisClient_HGet_Call) RunAndReturn(run func(context.Context, string, string) *redis.StringCmd) *MockRedisClient_HGet_Call {
	_c.Call.Return(run)
	return _c
}

// HScan provides a mock function with given fields: ctx, key, cursor, match, count
func (_m *MockRedisClient) HScan(ctx context.Context, key string, cursor uint64, match string, count int64) *redis.ScanCmd {
	ret := _m.Called(ctx, key, cursor, match, count)

	if len(ret) == 0 {
		panic("no return value specified for HScan")
	}

	var r0 *redis.ScanCmd
	if rf, ok := ret.Get(0).(func(context.Context, string, uint64, string, int64) *redis.ScanCmd); ok {
		r0 = rf(ctx, key, cursor, match, count)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).(*redis.ScanCmd)
		}
	}

	return r0
}

// MockRedisClient_HScan_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'HScan'
type MockRedisClient_HScan_Call struct {
	*mock.Call
}

// HScan is a helper method to define mock.On call
//   - ctx context.Context
//   - key string
//   - cursor uint64
//   - match string
//   - count int64
func (_e *MockRedisClient_Expecter) HScan(ctx interface{}, key interface{}, cursor interface{}, match interface{}, count interface{}) *MockRedisClient_HScan_Call {
	return &MockRedisClient_HScan_Call{Call: _e.mock.On("HScan", ctx, key, cursor, match, count)}
}

func (_c *MockRedisClient_HScan_Call) Run(run func(ctx context.Context, key string, cursor uint64, match string, count int64)) *MockRedisClient_HScan_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context), args[1].(string), args[2].(uint64), args[3].(string), args[4].(int64))
	})
	return _c
}

func (_c *MockRedisClient_HScan_Call) Return(_a0 *redis.ScanCmd) *MockRedisClient_HScan_Call {
	_c.Call.Return(_a0)
	return _c
}

func (_c *MockRedisClient_HScan_Call) RunAndReturn(run func(context.Context, string, uint64, string, int64) *redis.ScanCmd) *MockRedisClient_HScan_Call {
	_c.Call.Return(run)
	return _c
}

// HSet provides a mock function with given fields: ctx, key, values
func (_m *MockRedisClient) HSet(ctx context.Context, key string, values ...interface{}) *redis.IntCmd {
	var _ca []interface{}
	_ca = append(_ca, ctx, key)
	_ca = append(_ca, values...)
	ret := _m.Called(_ca...)

	if len(ret) == 0 {
		panic("no return value specified for HSet")
	}

	var r0 *redis.IntCmd
	if rf, ok := ret.Get(0).(func(context.Context, string, ...interface{}) *redis.IntCmd); ok {
		r0 = rf(ctx, key, values...)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).(*redis.IntCmd)
		}
	}

	return r0
}

// MockRedisClient_HSet_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'HSet'
type MockRedisClient_HSet_Call struct {
	*mock.Call
}

// HSet is a helper method to define mock.On call
//   - ctx context.Context
//   - key string
//   - values ...interface{}
func (_e *MockRedisClient_Expecter) HSet(ctx interface{}, key interface{}, values ...interface{}) *MockRedisClient_HSet_Call {
	return &MockRedisClient_HSet_Call{Call: _e.mock.On("HSet",
		append([]interface{}{ctx, key}, values...)...)}
}

func (_c *MockRedisClient_HSet_Call) Run(run func(ctx context.Context, key string, values ...interface{})) *MockRedisClient_HSet_Call {
	_c.Call.Run(func(args mock.Arguments) {
		variadicArgs := make([]interface{}, len(args)-2)
		for i, a := range args[2:] {
			if a != nil {
				variadicArgs[i] = a.(interface{})
			}
		}
		run(args[0].(context.Context), args[1].(string), variadicArgs...)
	})
	return _c
}

func (_c *MockRedisClient_HSet_Call) Return(_a0 *redis.IntCmd) *MockRedisClient_HSet_Call {
	_c.Call.Return(_a0)
	return _c
}

func (_c *MockRedisClient_HSet_Call) RunAndReturn(run func(context.Context, string, ...interface{}) *redis.IntCmd) *MockRedisClient_HSet_Call {
	_c.Call.Return(run)
	return _c
}

// Ping provides a mock function with given fields: ctx
func (_m *MockRedisClient) Ping(ctx context.Context) *redis.StatusCmd {
	ret := _m.Called(ctx)

	if len(ret) == 0 {
		panic("no return value specified for Ping")
	}

	var r0 *redis.StatusCmd
	if rf, ok := ret.Get(0).(func(context.Context) *redis.StatusCmd); ok {
		r0 = rf(ctx)
	} else {
		if ret.Get(0) != nil {
			r0 = ret.Get(0).(*redis.StatusCmd)
		}
	}

	return r0
}

// MockRedisClient_Ping_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'Ping'
type MockRedisClient_Ping_Call struct {
	*mock.Call
}

// Ping is a helper method to define mock.On call
//   - ctx context.Context
func (_e *MockRedisClient_Expecter) Ping(ctx interface{}) *MockRedisClient_Ping_Call {
	return &MockRedisClient_Ping_Call{Call: _e.mock.On("Ping", ctx)}
}

func (_c *MockRedisClient_Ping_Call) Run(run func(ctx context.Context)) *MockRedisClient_Ping_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(context.Context))
	})
	return _c
}

func (_c *MockRedisClient_Ping_Call) Return(_a0 *redis.StatusCmd) *MockRedisClient_Ping_Call {
	_c.Call.Return(_a0)
	return _c
}

func (_c *MockRedisClient_Ping_Call) RunAndReturn(run func(context.Context) *redis.StatusCmd) *MockRedisClient_Ping_Call {
	_c.Call.Return(run)
	return _c
}

// NewMockRedisClient creates a new instance of MockRedisClient. It also registers a testing interface on the mock and a cleanup function to assert the mocks expectations.
// The first argument is typically a *testing.T value.
func NewMockRedisClient(t interface {
	mock.TestingT
	Cleanup(func())
}) *MockRedisClient {
	mock := &MockRedisClient{}
	mock.Mock.Test(t)

	t.Cleanup(func() { mock.AssertExpectations(t) })

	return mock
}
