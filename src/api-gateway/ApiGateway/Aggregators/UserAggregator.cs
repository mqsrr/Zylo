using System.Buffers;
using System.Diagnostics.CodeAnalysis;
using System.Net;
using ApiGateway.Models;
using ApiGateway.Services;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using Ocelot.Middleware;
using Ocelot.Multiplexer;

namespace ApiGateway.Aggregators;

public sealed class UserAggregator : IDefinedAggregator
{
    private readonly IHttpClientFactory _clientFactory;

    public UserAggregator(IHttpClientFactory clientFactory)
    {
        _clientFactory = clientFactory;
    }

    [SuppressMessage("ReSharper", "ConditionIsAlwaysTrueOrFalseAccordingToNullableAPIContract")]
    public async Task<DownstreamResponse> Aggregate(List<HttpContext> responses)
    {
        var downstreamResponses = responses.Select(context => context.Items)
            .ToDictionary(items => items.DownstreamRoute().Key, items => items.DownstreamResponse());

        var user = await downstreamResponses["user-info"].Content.ReadFromJsonAsync<UserProfile>();
        var headers = downstreamResponses.SelectMany(r => r.Value.Headers).ToList();

        headers.Add(new Header("Content-Type", ["application/json"]));
        if (user is null)
        {
            return new DownstreamResponse(new StringContent("User cannot be found"), HttpStatusCode.NotFound, headers, "Not Found");
        }

        string authorizeHeader = responses.First().Request.Headers.Authorization!;
        var relationships = await downstreamResponses["user-relationships"].Content.ReadFromJsonAsync<UserRelationships>();

        var paginatedPosts = await downstreamResponses["user-posts"].Content.ReadFromJsonAsync<PaginatedResponse<Post>>();
        var replyTasksBuffer = ArrayPool<Task<PostInteractionResponse>>.Shared.Rent(paginatedPosts!.Data.Count);

        PostInteractionResponse[] replies;
        string userId = user.Id.ToString();
        
        var query = responses.Select(context => context.Request.Query).FirstOrDefault(query => query.ContainsKey("userId"));
        if (query is not null && query.TryGetValue("userId", out var userIdValue))
        {
            userId = userIdValue.ToString();
        }

        try
        {
            using var httpClient = _clientFactory.CreateClient();
            httpClient.DefaultRequestHeaders.Add("Authorization", authorizeHeader);

            for (int i = 0; i < paginatedPosts.Data.Count; i++)
            {
                replyTasksBuffer[i] = FetchRepliesForPost(paginatedPosts.Data[i].Id.ToString(), userId, httpClient);
            }

            replies = await Task.WhenAll(replyTasksBuffer.Where(task => task is not null));
        }
        finally
        {
            ArrayPool<Task<PostInteractionResponse>>.Shared.Return(replyTasksBuffer, true);
        }

        user.Relationships = relationships!;
        user.Posts = new PaginatedResponse<Post>
        {
            Data = PostInteractionMapper.MapPostInteractions(paginatedPosts.Data, replies),
            PerPage = paginatedPosts.PerPage,
            HasNextPage = paginatedPosts.HasNextPage,
            Next = paginatedPosts.Next
        };

        return new DownstreamResponse(new StringContent(JsonConvert.SerializeObject(user, new JsonSerializerSettings
        {
            NullValueHandling = NullValueHandling.Ignore,
            ContractResolver = new CamelCasePropertyNamesContractResolver()
        })), HttpStatusCode.OK, headers, "OK");
    }

    private static async Task<PostInteractionResponse> FetchRepliesForPost(string postId, string userId, HttpClient client)
    {
        var response = await client.GetAsync($"http://user-interaction:8080/api/posts/{postId}/replies?userId={userId}");
        return (await response.Content.ReadFromJsonAsync<PostInteractionResponse>())!;
    }
}