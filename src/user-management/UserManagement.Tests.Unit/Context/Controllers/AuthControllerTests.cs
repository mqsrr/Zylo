using AutoFixture;
using FluentAssertions;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using NSubstitute;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Controllers;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Controllers;

public sealed class AuthControllerTests
{
    private readonly AuthController _sut;
    private readonly IAuthService _authService;
    private readonly HttpContext _httpContext;
    private readonly Fixture _fixture;

    public AuthControllerTests()
    {
        _fixture = new Fixture();
        _fixture.Customize(new AuthenticationSuccessResultCustomization())
            .Customize(new RegisterRequestCustomization());

        _authService = Substitute.For<IAuthService>();
        _httpContext = Substitute.For<HttpContext>();
        _sut = new AuthController(_authService)
        {
            ControllerContext =
            {
                HttpContext = _httpContext
            }
        };

    }

    [Fact]
    public async Task Register_ShouldReturnOk_WhenRegistrationIsSuccessful()
    {
        // Arrange
        var request = _fixture.Create<RegisterRequest>();
        var authResult = _fixture.Create<AuthenticationResult>();
        var refreshToken = _fixture.Create<RefreshTokenResponse>();
        var cancellationToken = CancellationToken.None;
        
        _authService.RegisterAsync(request, cancellationToken)
            .Returns((authResult, refreshToken));

        // Act
        var result = await _sut.Register(request, cancellationToken);

        // Assert
        result.Should().BeOfType<OkObjectResult>();
        
        var okResult = result as OkObjectResult;
        okResult.Should().NotBeNull();
        okResult!.Value.Should().BeEquivalentTo(authResult);

        await _authService.Received().RegisterAsync(request, cancellationToken);
    }

    [Fact]
    public async Task Register_ShouldReturnBadRequest_WhenRegistrationFails()
    {
        // Arrange
        var fixture = new Fixture().Customize(new AuthenticationFailureResultCustomization()).Customize(new IFormFileCustomization()).Customize(new DateOnlyCustomization());
        var request = fixture.Create<RegisterRequest>();
        var authResult = fixture.Create<AuthenticationResult>();
        var cancellationToken = CancellationToken.None;
        
        _authService.RegisterAsync(request, cancellationToken)
            .Returns((authResult, null));

        // Act
        var result = await _sut.Register(request, cancellationToken);

        // Assert
        result.Should().BeOfType<BadRequestObjectResult>();
        
        var badRequestResult = result as BadRequestObjectResult;
        badRequestResult.Should().NotBeNull();
        badRequestResult!.Value.Should().BeEquivalentTo(authResult);

        await _authService.Received().RegisterAsync(request, cancellationToken);
    }

    [Fact]
    public async Task Login_ShouldReturnOk_WhenLoginIsSuccessful()
    {
        // Arrange
        var request = _fixture.Create<LoginRequest>();
        var authResult = _fixture.Create<AuthenticationResult>();
        var refreshToken = _fixture.Create<RefreshTokenResponse>();
        var cancellationToken = CancellationToken.None;
        
        _authService.LoginAsync(request, cancellationToken)
            .Returns((authResult, refreshToken));

        // Act
        var result = await _sut.Login(request, cancellationToken);

        // Assert
        result.Should().BeOfType<OkObjectResult>();
        
        var okResult = result as OkObjectResult;
        okResult.Should().NotBeNull();
        okResult!.Value.Should().BeEquivalentTo(authResult);

        await _authService.Received().LoginAsync(request, cancellationToken);
    }

    [Fact]
    public async Task Login_ShouldReturnBadRequest_WhenLoginFails()
    {
        // Arrange
        var fixture = new Fixture().Customize(new AuthenticationFailureResultCustomization());
        
        var request = fixture.Create<LoginRequest>();
        var authResult = fixture.Create<AuthenticationResult>();
        var cancellationToken = CancellationToken.None;
        
        _authService.LoginAsync(request, cancellationToken)
            .Returns((authResult, null));

        // Act
        var result = await _sut.Login(request,cancellationToken);

        // Assert
        result.Should().BeOfType<BadRequestObjectResult>();
        
        var badRequestResult = result as BadRequestObjectResult;
        badRequestResult.Should().NotBeNull();
        badRequestResult!.Value.Should().BeEquivalentTo(authResult);

        await _authService.Received().LoginAsync(request, cancellationToken);
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnOk_WhenRefreshTokenIsValid()
    {
        // Arrange
        var authResult = _fixture.Create<AuthenticationResult>();
        var refreshToken = _fixture.Create<RefreshTokenResponse>();
        var cancellationToken = CancellationToken.None;
        
        _httpContext.Request.Cookies["refresh-token"].Returns(refreshToken.Value);
        _authService.RefreshAccessToken(Arg.Is(refreshToken.Value), cancellationToken)
            .Returns((authResult, refreshToken));

        // Act
        var result = await _sut.RefreshAccessToken(cancellationToken);

        // Assert
        result.Should().BeOfType<OkObjectResult>();
        
        var okResult = result as OkObjectResult;
        okResult.Should().NotBeNull();
        okResult!.Value.Should().BeEquivalentTo(authResult);

        await _authService.Received().RefreshAccessToken(Arg.Is(refreshToken.Value), cancellationToken);
        _ = _httpContext.Request.Received().Cookies["refresh-token"];
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnBadRequest_WhenRefreshTokenIsInvalid()
    {
        // Arrange
        var fixture = new Fixture().Customize(new AuthenticationFailureResultCustomization());
        var authResult = fixture.Create<AuthenticationResult>();
        var cancellationToken = CancellationToken.None;
        
        _httpContext.Request.Cookies["refresh-token"].Returns("invalid-token");
        _authService.RefreshAccessToken("invalid-token", cancellationToken)
            .Returns((authResult, null));

        // Act
        var result = await _sut.RefreshAccessToken(cancellationToken);

        // Assert
        result.Should().BeOfType<BadRequestObjectResult>();
        
        var badRequestResult = result as BadRequestObjectResult;
        badRequestResult.Should().NotBeNull();
        badRequestResult!.Value.Should().BeEquivalentTo(authResult);

        await _authService.Received().RefreshAccessToken(Arg.Is("invalid-token"), cancellationToken);
        _ = _httpContext.Request.Received().Cookies["refresh-token"];
    }

    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnNoContent_WhenRevocationIsSuccessful()
    {
        // Arrange
        var cancellationToken = CancellationToken.None;
        
        _httpContext.Request.Cookies["refresh-token"].Returns("valid-token");
        _authService.RevokeRefreshToken("valid-token", cancellationToken)
            .Returns(true);

        // Act
        var result = await _sut.RevokeRefreshToken(cancellationToken);

        // Assert
        result.Should().BeOfType<NoContentResult>();

        await _authService.Received().RevokeRefreshToken(Arg.Is("valid-token"), cancellationToken);
        _ = _httpContext.Request.Received().Cookies["refresh-token"];
    }

    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnNotFound_WhenRevocationFails()
    {
        // Arrange
        var cancellationToken = CancellationToken.None;
        
        _httpContext.Request.Cookies["refresh-token"].Returns("invalid-token");
        _authService.RevokeRefreshToken("invalid-token", cancellationToken)
            .Returns(false);

        // Act
        var result = await _sut.RevokeRefreshToken(cancellationToken);

        // Assert
        result.Should().BeOfType<NotFoundResult>();

        await _authService.Received().RevokeRefreshToken(Arg.Is("invalid-token"), cancellationToken);
        _ =  _httpContext.Request.Received().Cookies["refresh-token"];
    }
}