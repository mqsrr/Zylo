
using AutoFixture;
using FluentAssertions;
using Newtonsoft.Json;
using NSubstitute;
using StackExchange.Redis;
using UserManagement.Application.Services;
// ReSharper disable UnusedMember.Local

namespace UserManagement.Tests.Unit.Context.Services;

public class CacheServiceTests
{
    private readonly CacheService _sut;
    private readonly IDatabase _database;
    private readonly Fixture _fixture;

    public CacheServiceTests()
    {
        _fixture = new Fixture();
        var connection = Substitute.For<IConnectionMultiplexer>();
        _database = Substitute.For<IDatabase>();
        connection.GetDatabase().Returns(_database);

        _sut = new CacheService(connection);
    }

    [Fact]
    public async Task HGetAsync_ShouldReturnEntity_WhenEntityExistsInCache()
    {
        // Arrange
        const string key = "test-key";
        const string field = "test-field";

        var entity = _fixture.Create<TestEntity>();
        string cachedEntityJson = JsonConvert.SerializeObject(entity);

        _database.HashGetAsync(key, field).Returns(cachedEntityJson);

        // Act
        var result = await _sut.HGetAsync<TestEntity>(key, field);

        // Assert
        result.Should().BeEquivalentTo(entity);
        await _database.Received().HashGetAsync(key, field);
    }

    [Fact]
    public async Task HGetAsync_ShouldReturnNull_WhenEntityDoesNotExistInCache()
    {
        // Arrange
        const string key = "test-key";
        const string field = "test-field";

        _database.HashGetAsync(key, field).Returns(RedisValue.Null);

        // Act
        var result = await _sut.HGetAsync<TestEntity>(key, field);

        // Assert
        result.Should().BeNull();
        await _database.Received().HashGetAsync(key, field);
    }

    [Fact]
    public async Task HSetAsync_ShouldStoreEntityInCache_WithExpiry()
    {
        // Arrange
        const string key = "test-key";
        const string field = "test-field";
        
        string entityJson = JsonConvert.SerializeObject(_fixture.Create<TestEntity>());
        var expiry = TimeSpan.FromMinutes(5);

        // Act
        await _sut.HSetAsync(key, field, entityJson, expiry);

        // Assert
        await _database.Received().HashSetAsync(key, field, entityJson);
        await _database.Received().HashFieldExpireAsync(key, Arg.Is<RedisValue[]>(array => array.Contains(field)), expiry);
    }

    [Fact]
    public async Task HRemoveAsync_ShouldDeleteEntityFromCache()
    {
        // Arrange
        const string key = "test-key";
        const string field = "test-field";

        // Act
        await _sut.HRemoveAsync(key, field);

        // Assert
        await _database.Received().HashDeleteAsync(key, field);
    }

    [Fact]
    public async Task GetOrCreateAsync_ShouldReturnCachedEntity_WhenEntityExistsInCache()
    {
        // Arrange
        const string key = "test-key";
        const string field = "test-field";
        var cachedEntity = _fixture.Create<TestEntity>();
        
        _database.HashGetAsync(key, field).Returns(JsonConvert.SerializeObject(cachedEntity));

        // Act
        var result = await _sut.GetOrCreateAsync(key, field, () => Task.FromResult<TestEntity?>(null), TimeSpan.FromMinutes(5));

        // Assert
        result.Should().BeEquivalentTo(cachedEntity);
        await _database.Received().HashGetAsync(key, field);
        await _database.DidNotReceiveWithAnyArgs().HashSetAsync(default, default!);
    }

    [Fact]
    public async Task GetOrCreateAsync_ShouldReturnCreatedAndCacheEntity_WhenEntityDoesNotExistInCache()
    {
        // Arrange
        const string key = "test-key";
        const string field = "test-field";
        
        var newEntity = _fixture.Create<TestEntity>();
        string newEntityJson = JsonConvert.SerializeObject(newEntity);
        var expiry = TimeSpan.FromMinutes(10);

        _database.HashGetAsync(key, field).Returns(RedisValue.Null);
        
        // Act
        var result = await _sut.GetOrCreateAsync(key, field, () => Task.FromResult<TestEntity?>(newEntity), expiry);

        // Assert
        result.Should().BeEquivalentTo(newEntity);
        await _database.Received().HashSetAsync(key, field, newEntityJson);
        await _database.Received().HashFieldExpireAsync(key, Arg.Is<RedisValue[]>(array => array.Contains(field)), expiry);
    }
    
    [Fact]
    public async Task GetOrCreateAsync_ShouldReturnNull_WhenEntityDoesNotExistInCache_AndCouldNotBeCreated()
    {
        // Arrange
        const string key = "test-key";
        const string field = "test-field";
        
        var expiry = TimeSpan.FromMinutes(10);

        _database.HashGetAsync(key, field).Returns(RedisValue.Null);
        
        // Act
        var result = await _sut.GetOrCreateAsync(key, field, () => Task.FromResult<TestEntity?>(null), expiry);

        // Assert
        result.Should().BeNull();
        await _database.DidNotReceiveWithAnyArgs().HashSetAsync(default, default!);
        await _database.DidNotReceiveWithAnyArgs().HashFieldExpireAsync(default, Arg.Is<RedisValue[]>(array => array.Contains(field)), expiry);
    }

    private class TestEntity
    {
        public Guid Id { get; set; }

        public string? Name { get; set; }
    }
}