// Code generated by mockery v2.46.0. DO NOT EDIT.

package middleware

import (
	http "net/http"

	mock "github.com/stretchr/testify/mock"
)

// MockErrHandlerFunc is an autogenerated mock type for the ErrHandlerFunc type
type MockErrHandlerFunc struct {
	mock.Mock
}

type MockErrHandlerFunc_Expecter struct {
	mock *mock.Mock
}

func (_m *MockErrHandlerFunc) EXPECT() *MockErrHandlerFunc_Expecter {
	return &MockErrHandlerFunc_Expecter{mock: &_m.Mock}
}

// Execute provides a mock function with given fields: w, r
func (_m *MockErrHandlerFunc) Execute(w http.ResponseWriter, r *http.Request) error {
	ret := _m.Called(w, r)

	if len(ret) == 0 {
		panic("no return value specified for Execute")
	}

	var r0 error
	if rf, ok := ret.Get(0).(func(http.ResponseWriter, *http.Request) error); ok {
		r0 = rf(w, r)
	} else {
		r0 = ret.Error(0)
	}

	return r0
}

// MockErrHandlerFunc_Execute_Call is a *mock.Call that shadows Run/Return methods with type explicit version for method 'Execute'
type MockErrHandlerFunc_Execute_Call struct {
	*mock.Call
}

// Execute is a helper method to define mock.On call
//   - w http.ResponseWriter
//   - r *http.Request
func (_e *MockErrHandlerFunc_Expecter) Execute(w interface{}, r interface{}) *MockErrHandlerFunc_Execute_Call {
	return &MockErrHandlerFunc_Execute_Call{Call: _e.mock.On("Execute", w, r)}
}

func (_c *MockErrHandlerFunc_Execute_Call) Run(run func(w http.ResponseWriter, r *http.Request)) *MockErrHandlerFunc_Execute_Call {
	_c.Call.Run(func(args mock.Arguments) {
		run(args[0].(http.ResponseWriter), args[1].(*http.Request))
	})
	return _c
}

func (_c *MockErrHandlerFunc_Execute_Call) Return(_a0 error) *MockErrHandlerFunc_Execute_Call {
	_c.Call.Return(_a0)
	return _c
}

func (_c *MockErrHandlerFunc_Execute_Call) RunAndReturn(run func(http.ResponseWriter, *http.Request) error) *MockErrHandlerFunc_Execute_Call {
	_c.Call.Return(run)
	return _c
}

// NewMockErrHandlerFunc creates a new instance of MockErrHandlerFunc. It also registers a testing interface on the mock and a cleanup function to assert the mocks expectations.
// The first argument is typically a *testing.T value.
func NewMockErrHandlerFunc(t interface {
	mock.TestingT
	Cleanup(func())
}) *MockErrHandlerFunc {
	mock := &MockErrHandlerFunc{}
	mock.Mock.Test(t)

	t.Cleanup(func() { mock.AssertExpectations(t) })

	return mock
}
