using System.Net;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using Amazon.S3.Model;
using AutoBogus;
using AutoFixture;
using Dapper;
using FluentAssertions;
using Microsoft.Extensions.DependencyInjection;
using NSubstitute;
using NSubstitute.ClearExtensions;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Contracts.Responses;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Tests.Integration.Fixtures;
using UserManagement.Tests.Integration.Fixtures.Customizations;
using UserManagement.Tests.Integration.Fixtures.Fakers;

namespace UserManagement.Tests.Integration.Context.Controllers;

[Collection(nameof(UserManagementApiCollection))]
public sealed class UsersControllerTests : IAsyncDisposable
{

    private readonly UserManagementApiFactory _factory;
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly Fixture _fixture;
    private readonly AsyncServiceScope _serviceScope;

    public UsersControllerTests(UserManagementApiFactory factory)
    {
        _fixture = new Fixture();
        _fixture.Customize(new UserCustomization())
            .Customize(new UpdateUserRequestCustomization());
        
        _factory = factory;
        _serviceScope = factory.Services.CreateAsyncScope();
        _dbConnectionFactory = _serviceScope.ServiceProvider.GetRequiredService<IDbConnectionFactory>();
    }
    
    
    [Fact]
    public async Task GetById_ShouldReturnOk_WithUserDetails_WhenUserExists()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var user = _fixture.Create<User>();
        var expectedResponse = user.ToResponse();
        
        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await connection.ExecuteAsync(SqlQueries.Users.Create, user);
        var getObjectMetadataResponse = new GetObjectMetadataResponse
        {
            HttpStatusCode = HttpStatusCode.OK,
            Metadata =
            {
                ["Content-Type"] = "image/jpeg",
                ["file-name"] = "profile.jpg"
            }
        };

        _factory.S3.GetPreSignedURLAsync(default)
            .ReturnsForAnyArgs("mocked");        
        
        _factory.S3.GetObjectMetadataAsync(default)
            .ReturnsForAnyArgs(getObjectMetadataResponse);
        
        using var httpClient = _factory.CreateClient();

        // Act
        var response = await httpClient.GetAsync(ApiEndpoints.Users.GetById.Replace("{id}", user.Id.Value.ToString()));

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.OK);

        var userResponse = await response.Content.ReadFromJsonAsync<UserResponse>();
        userResponse.Should().NotBeNull();
        userResponse.Should().BeEquivalentTo(expectedResponse, options => 
            options.Excluding(r  => r.ProfileImage).Excluding(r => r.BackgroundImage));

        await _factory.S3.ReceivedWithAnyArgs(2).GetPreSignedURLAsync(default);
        await _factory.S3.ReceivedWithAnyArgs(2).GetObjectMetadataAsync(default);
    }

    [Fact]
    public async Task GetById_ShouldReturnNotFound_WhenUserDoesNotExist()
    {
        // Arrange
        var userId = UserId.NewId();
        using var httpClient = _factory.CreateClient();

        // Act
        var response = await httpClient.GetAsync(ApiEndpoints.Users.GetById.Replace("{id}", userId.Value.ToString()));

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.NotFound);
    }

    [Fact]
    public async Task Update_ShouldReturnNoContent_WhenUserIsSuccessfullyUpdated()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var user = _fixture.Create<User>();
        var updateRequest = AutoFaker.Generate<UpdateUserRequest, UpdateUserRequestFaker>();
        updateRequest.Id = user.Id;

        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await connection.ExecuteAsync(SqlQueries.Users.Create, user);

        using var httpClient = _factory.CreateClient();
        using var formData = new MultipartFormDataContent();

        formData.Add(new StringContent(updateRequest.Name), "Name");
        formData.Add(new StringContent(updateRequest.Bio), "Bio");
        formData.Add(new StringContent(updateRequest.Location), "Location");
        formData.Add(new StringContent(updateRequest.BirthDate.ToString()), "BirthDate");
        formData.Add(
            new StreamContent(updateRequest.ProfileImage.OpenReadStream())
                {Headers = {ContentType = new MediaTypeHeaderValue(updateRequest.ProfileImage.ContentType)}}, "ProfileImage",
            updateRequest.ProfileImage.FileName);

        formData.Add(
            new StreamContent(updateRequest.BackgroundImage.OpenReadStream())
                {Headers = {ContentType = new MediaTypeHeaderValue(updateRequest.BackgroundImage.ContentType)}}, "BackgroundImage",
            updateRequest.BackgroundImage.FileName);
        
        var putObjectResponse = new PutObjectResponse
        {
            HttpStatusCode = HttpStatusCode.OK,
        };

        _factory.S3.PutObjectAsync(default)
            .ReturnsForAnyArgs(putObjectResponse);
        
        // Act
        var response = await httpClient.PutAsync(ApiEndpoints.Users.Update.Replace("{id}", user.Id.Value.ToString()), formData);
        
        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.NoContent);

        var updatedUser = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new { user.Id });
        updatedUser.Should().NotBeNull();
        updatedUser.Should().BeEquivalentTo(updateRequest, options => options.ExcludingMissingMembers().Excluding(r => r.ProfileImage).Excluding(r => r.BackgroundImage));

        await _factory.S3.ReceivedWithAnyArgs(2).PutObjectAsync(default);
    }

    [Fact]
    public async Task Update_ShouldReturnBadRequest_WhenUpdateFails()
    {
        // Arrange
        var userId = UserId.NewId();
        var updateRequest = AutoFaker.Generate<UpdateUserRequest, UpdateUserRequestFaker>();
        updateRequest.Id = userId;

        using var httpClient = _factory.CreateClient();

        using var formData = new MultipartFormDataContent();

        formData.Add(new StringContent(updateRequest.Name), "Name");
        formData.Add(new StringContent(updateRequest.Bio), "Bio");
        formData.Add(new StringContent(updateRequest.Location), "Location");
        formData.Add(new StringContent(updateRequest.BirthDate.ToString()), "BirthDate");

        // Act
        var response = await httpClient.PutAsync(ApiEndpoints.Users.Update.Replace("{id}", userId.Value.ToString()), formData);

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.BadRequest);
    }

    [Fact]
    public async Task DeleteById_ShouldReturnNoContent_WhenUserIsSuccessfullyDeleted()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var user = _fixture.Create<User>();
        
        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await connection.ExecuteAsync(SqlQueries.Users.Create, user);

        var objectMetadataResponse = new DeleteObjectsResponse
        {
            HttpStatusCode = HttpStatusCode.OK
        };
        
        _factory.S3.DeleteObjectsAsync(default)
            .ReturnsForAnyArgs(objectMetadataResponse);
        
        using var httpClient = _factory.CreateClient();

        // Act
        var response = await httpClient.DeleteAsync(ApiEndpoints.Users.DeleteById.Replace("{id}", user.Id.Value.ToString()));

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.NoContent);

        var deletedUser = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new { user.Id });
        deletedUser.Should().BeNull();

        await _factory.S3.ReceivedWithAnyArgs().DeleteObjectsAsync(default);
    }

    [Fact]
    public async Task DeleteById_ShouldReturnNotFound_WhenUserDoesNotExist()
    {
        // Arrange
        var userId = UserId.NewId();
        using var httpClient = _factory.CreateClient();

        // Act
        var response = await httpClient.DeleteAsync(ApiEndpoints.Users.DeleteById.Replace("{id}", userId.Value.ToString()));

        // Assert
        response.StatusCode.Should().Be(HttpStatusCode.NotFound);
    }
    public async ValueTask DisposeAsync()
    {
        await _serviceScope.DisposeAsync();
    }
}