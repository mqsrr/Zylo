using System.Net;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using Amazon.S3.Model;
using AutoBogus;
using AutoFixture;
using Dapper;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing.Handlers;
using Microsoft.Extensions.DependencyInjection;
using NSubstitute;
using NSubstitute.ClearExtensions;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Tests.Integration.Fixtures;
using UserManagement.Tests.Integration.Fixtures.Customizations;
using UserManagement.Tests.Integration.Fixtures.Fakers;

namespace UserManagement.Tests.Integration.Context.Controllers;

[Collection(nameof(UserManagementApiCollection))]
public sealed class AuthControllerTests : IAsyncDisposable
{
    private readonly UserManagementApiFactory _factory;
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly Fixture _fixture;
    private readonly AsyncServiceScope _serviceScope;

    public AuthControllerTests(UserManagementApiFactory factory)
    {
        _fixture = new Fixture();
        _fixture.Customize(new RegisterRequestCustomization());

        _factory = factory;
        _serviceScope = factory.Services.CreateAsyncScope();
        _dbConnectionFactory = _serviceScope.ServiceProvider.GetRequiredService<IDbConnectionFactory>();
    }

    [Fact]
    public async Task Register_ShouldReturnOk_WithAuthResultAndSetCookie_WhenRegisterIsSuccessful()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var registerRequest = AutoFaker.Generate<RegisterRequest, RegisterRequestFaker>();
        using var httpClient = _factory.CreateClient();

        using var formData = new MultipartFormDataContent();

        formData.Add(new StringContent(registerRequest.Username), "Username");
        formData.Add(new StringContent(registerRequest.Password), "Password");
        formData.Add(new StringContent(registerRequest.Email), "Email");
        formData.Add(new StringContent(registerRequest.Name), "Name");
        formData.Add(new StringContent(registerRequest.Bio ?? string.Empty), "Bio");
        formData.Add(new StringContent(registerRequest.Location ?? string.Empty), "Location");
        formData.Add(new StringContent(registerRequest.BirthDate.ToString()), "BirthDate");
        formData.Add(
            new StreamContent(registerRequest.ProfileImage.OpenReadStream())
                {Headers = {ContentType = new MediaTypeHeaderValue(registerRequest.ProfileImage.ContentType)}}, "ProfileImage",
            registerRequest.ProfileImage.FileName);

        formData.Add(
            new StreamContent(registerRequest.BackgroundImage.OpenReadStream())
                {Headers = {ContentType = new MediaTypeHeaderValue(registerRequest.BackgroundImage.ContentType)}}, "BackgroundImage",
            registerRequest.BackgroundImage.FileName);
        
        var cancellationToken = CancellationToken.None;
        var objectResponse = new PutObjectResponse
        {
            HttpStatusCode = HttpStatusCode.OK
        };

        _factory.S3.PutObjectAsync(default, cancellationToken)
            .ReturnsForAnyArgs(objectResponse);
        
        // Act
        var response = await httpClient.PostAsync(ApiEndpoints.Authentication.Register, formData,  cancellationToken);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.OK);

        var authResult = await response.Content.ReadFromJsonAsync<AuthenticationResult>(cancellationToken);
        authResult.Should().NotBeNull();
        authResult!.Success.Should().BeTrue();

        response.Headers.Should().Contain(header => header.Key == "Set-Cookie");
        string? cookie = response.Headers.GetValues("Set-Cookie").FirstOrDefault();
        cookie.Should().Contain("refresh-token=");

        await _factory.S3.ReceivedWithAnyArgs(2).PutObjectAsync(default, cancellationToken);
    }

    [Fact]
    public async Task Register_ShouldReturnBadRequest_WhenRegisterFails()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var registerRequest = AutoFaker.Generate<RegisterRequest, RegisterRequestFaker>();
        using var httpClient = _factory.CreateClient();

        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await connection.ExecuteAsync(SqlQueries.Authentication.Register, registerRequest.ToIdentity());

        using var formData = new MultipartFormDataContent();

        formData.Add(new StringContent(registerRequest.Username), "Username");
        formData.Add(new StringContent(registerRequest.Password), "Password");
        formData.Add(new StringContent(registerRequest.Email), "Email");
        formData.Add(new StringContent(registerRequest.Name), "Name");
        formData.Add(new StringContent(registerRequest.Bio ?? string.Empty), "Bio");
        formData.Add(new StringContent(registerRequest.Location ?? string.Empty), "Location");
        formData.Add(new StringContent(registerRequest.BirthDate.ToString()), "BirthDate");
        formData.Add(
            new StreamContent(registerRequest.ProfileImage.OpenReadStream())
                {Headers = {ContentType = new MediaTypeHeaderValue(registerRequest.ProfileImage.ContentType)}}, "ProfileImage",
            registerRequest.ProfileImage.FileName);

        formData.Add(
            new StreamContent(registerRequest.BackgroundImage.OpenReadStream())
                {Headers = {ContentType = new MediaTypeHeaderValue(registerRequest.BackgroundImage.ContentType)}}, "BackgroundImage",
            registerRequest.BackgroundImage.FileName);
        // Act

        var response = await httpClient.PostAsync(ApiEndpoints.Authentication.Register, formData);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.BadRequest);

        var authResult = await response.Content.ReadFromJsonAsync<AuthenticationResult>();
        authResult.Should().NotBeNull();

        authResult!.Success.Should().BeFalse();
        authResult.Error.Should().NotBeEmpty();
    }

    [Fact]
    public async Task Login_ShouldReturnOk_WithAuthResultAndSetCookie_WhenLoginIsSuccessful()
    {
        // Arrange
        var registerRequest = AutoFaker.Generate<RegisterRequest, RegisterRequestFaker>();
        var loginRequest = _fixture.Build<LoginRequest>()
            .With(r => r.Username, registerRequest.Username)
            .With(r => r.Password, registerRequest.Password)
            .Create();

        using var httpClient = _factory.CreateClient();

        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await connection.ExecuteAsync(SqlQueries.Authentication.Register, registerRequest.ToIdentity());

        // Act
        var response = await httpClient.PostAsJsonAsync(ApiEndpoints.Authentication.Login, loginRequest);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.OK);

        var authResult = await response.Content.ReadFromJsonAsync<AuthenticationResult>();
        authResult.Should().NotBeNull();
        authResult!.Success.Should().BeTrue();

        response.Headers.Should().Contain(header => header.Key == "Set-Cookie");
        string? cookie = response.Headers.GetValues("Set-Cookie").FirstOrDefault();
        cookie.Should().Contain("refresh-token=");
    }

    [Fact]
    public async Task Login_ShouldReturnBadRequest_WhenLoginFails()
    {
        // Arrange
        using var httpClient = _factory.CreateClient();
        var loginRequest = new LoginRequest
        {
            Username = "test",
            Password = "TestingPassword!"
        };

        // Act
        var response = await httpClient.PostAsJsonAsync(ApiEndpoints.Authentication.Login, loginRequest);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.BadRequest);

        var authResult = await response.Content.ReadFromJsonAsync<AuthenticationResult>();
        authResult.Should().NotBeNull();

        authResult!.Success.Should().BeFalse();
        authResult.Error.Should().NotBeEmpty();
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnOk_WithRefreshedAccessedToken_WhenRefreshIsSuccessful()
    {
        // Arrange
        var registerRequest = AutoFaker.Generate<RegisterRequest, RegisterRequestFaker>();
        var loginRequest = _fixture.Build<LoginRequest>()
            .With(r => r.Username, registerRequest.Username)
            .With(r => r.Password, registerRequest.Password)
            .Create();

        var cookieContainer = new CookieContainer();
        using var httpClient = _factory.CreateDefaultClient(_factory.Server.BaseAddress, new CookieContainerHandler(cookieContainer));

        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await connection.ExecuteAsync(SqlQueries.Authentication.Register, registerRequest.ToIdentity());
        
        
        // Act
        var loginResponse = await httpClient.PostAsJsonAsync(ApiEndpoints.Authentication.Login, loginRequest);
        
        var setCookieHeader = loginResponse.Headers.FirstOrDefault(header => header.Key == "Set-Cookie").Value;
        string refreshTokenCookie = setCookieHeader
            .Select(headerValue => headerValue.Split(';')[0])
            .FirstOrDefault(cookie => cookie.StartsWith("refresh-token=", StringComparison.OrdinalIgnoreCase))!
            .Split('=')[1];
        
        cookieContainer.Add(new Cookie("refresh-token", refreshTokenCookie)
        {
            Domain = _factory.Server.BaseAddress.Host
        });
        
        var response = await httpClient.PostAsync(ApiEndpoints.Authentication.RefreshAccessToken, null);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.OK);

        var authResult = await response.Content.ReadFromJsonAsync<AuthenticationResult>();
        authResult.Should().NotBeNull();
        authResult!.Success.Should().BeTrue();

        response.Headers.Should().Contain(header => header.Key == "Set-Cookie");
        string? cookie = response.Headers.GetValues("Set-Cookie").FirstOrDefault();
        cookie.Should().Contain("refresh-token=");
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnBadRequest_WhenRefreshFails()
    {
        // Arrange
        using var httpClient = _factory.CreateClient();

        // Act
        var response = await httpClient.PostAsync(ApiEndpoints.Authentication.RefreshAccessToken, null);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.BadRequest);

        var authResult = await response.Content.ReadFromJsonAsync<AuthenticationResult>();
        authResult.Should().NotBeNull();

        authResult!.Success.Should().BeFalse();
        authResult.Error.Should().NotBeEmpty();
    }

    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnOk_AndUnsetRefreshToken_WhenRevokeIsSuccessful()
    {
        // Arrange
        var registerRequest = AutoFaker.Generate<RegisterRequest, RegisterRequestFaker>();
        var loginRequest = _fixture.Build<LoginRequest>()
            .With(r => r.Username, registerRequest.Username)
            .With(r => r.Password, registerRequest.Password)
            .Create();

        var cookieContainer = new CookieContainer();
        using var httpClient = _factory.CreateDefaultClient(_factory.Server.BaseAddress, new CookieContainerHandler(cookieContainer));

        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await connection.ExecuteAsync(SqlQueries.Authentication.Register, registerRequest.ToIdentity());

        // Act
        var loginResponse = await httpClient.PostAsJsonAsync(ApiEndpoints.Authentication.Login, loginRequest);
        
        var setCookieHeader = loginResponse.Headers.FirstOrDefault(header => header.Key == "Set-Cookie").Value;
        string refreshTokenCookie = setCookieHeader
            .Select(headerValue => headerValue.Split(';')[0])
            .FirstOrDefault(cookie => cookie.StartsWith("refresh-token=", StringComparison.OrdinalIgnoreCase))!
            .Split('=')[1];
        
        cookieContainer.Add(new Cookie("refresh-token", refreshTokenCookie)
        {
            Domain = _factory.Server.BaseAddress.Host
        });
        
        var response = await httpClient.PostAsync(ApiEndpoints.Authentication.RevokeRefreshToken, null);
        
        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.NoContent);
        response.Headers.Should().Contain(header => header.Key == "Set-Cookie");
        string? cookie = response.Headers.GetValues("Set-Cookie").FirstOrDefault();
        cookie.Should().Contain("refresh-token=");
        
        setCookieHeader = response.Headers.FirstOrDefault(header => header.Key == "Set-Cookie").Value;
        setCookieHeader
            .Select(headerValue => headerValue.Split(';')[0])
            .FirstOrDefault(s => s.StartsWith("refresh-token=", StringComparison.OrdinalIgnoreCase))!
            .Split('=')[1]
            .Should().BeNullOrEmpty();
    }
    
    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnBadRequest_WhenRevokeFails()
    {
        // Arrange
        using var httpClient = _factory.CreateClient();

        // Act
        var response = await httpClient.PostAsync(ApiEndpoints.Authentication.RevokeRefreshToken, null);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.NotFound);
    }

    public async ValueTask DisposeAsync()
    {
        await _serviceScope.DisposeAsync();
    }
}