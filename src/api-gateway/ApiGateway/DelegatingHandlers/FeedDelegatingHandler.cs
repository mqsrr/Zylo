using System.Buffers;
using System.Diagnostics.CodeAnalysis;
using System.Net;
using System.Text;
using ApiGateway.Models;
using ApiGateway.Services;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

namespace ApiGateway.DelegatingHandlers;

public sealed class FeedDelegatingHandler : DelegatingHandler
{
    private readonly IHttpClientFactory _clientFactory;
    
    public FeedDelegatingHandler(IHttpClientFactory clientFactory)
    {
        _clientFactory = clientFactory;
    }

    [SuppressMessage("ReSharper", "ConditionIsAlwaysTrueOrFalseAccordingToNullableAPIContract")]
    protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
    {
        var downstreamResponse = await base.SendAsync(request, cancellationToken);
        if (!downstreamResponse.IsSuccessStatusCode)
        {
            return downstreamResponse;
        }

        string userId = GetUserIdFromUriPath(request.RequestUri!.PathAndQuery);
        var response = await downstreamResponse.Content.ReadFromJsonAsync<PaginatedResponse<string>>(cancellationToken: cancellationToken);
        if (response?.Data is null || response.Data.Count < 1)
        {
            return new HttpResponseMessage
            {
                RequestMessage = request,
                StatusCode = HttpStatusCode.NoContent,
                ReasonPhrase = "No Content"
            };
        }

        string authorizeHeader = request.Headers.Authorization!.ToString();
        var postsIds = response.Data;
        
        var postsTaskArray = ArrayPool<Task<Post>>.Shared.Rent(postsIds.Count);
        var repliesTaskArray = ArrayPool<Task<PostInteractionResponse>>.Shared.Rent(postsIds.Count);
        
        var httpClient = _clientFactory.CreateClient();
        httpClient.DefaultRequestHeaders.Add("Authorization", authorizeHeader);
        
        Post[] posts;
        PostInteractionResponse[] replies;

        try
        {
            for (int i = 0; i < postsIds.Count; i++)
            {
                postsTaskArray[i] = FetchPost(postsIds[i], httpClient);
                repliesTaskArray[i] = FetchRepliesForPost(postsIds[i], userId, httpClient);
            }
            
            posts = await Task.WhenAll(postsTaskArray.Where(task => task is not null));
            replies = await Task.WhenAll(repliesTaskArray.Where(task => task is not null));
        }
        finally
        {
            ArrayPool<Task<Post>>.Shared.Return(postsTaskArray, true);
            ArrayPool<Task<PostInteractionResponse>>.Shared.Return(repliesTaskArray, true);
        }

        var mappedPosts = PostInteractionMapper.MapPostInteractions(posts, replies);
        string aggregatedContent = JsonConvert.SerializeObject(new PaginatedResponse<Post>
        {
            Data = mappedPosts,
            PerPage = response.PerPage,
            HasNextPage = response.HasNextPage,
            Next = response.Next
        }, new JsonSerializerSettings
        {
            NullValueHandling = NullValueHandling.Ignore,
            ContractResolver = new CamelCasePropertyNamesContractResolver()
        });
        
        var stringContent = new StringContent(aggregatedContent, Encoding.UTF8, "application/json");
        var httpResponse = new HttpResponseMessage
        {
            Content = stringContent,
            ReasonPhrase = "Ok",
            RequestMessage = request,
            StatusCode = HttpStatusCode.OK,
        };
        
        return httpResponse;
    }

    private static string GetUserIdFromUriPath(ReadOnlySpan<char> requestPath)
    {
        Span<Range> ranges = stackalloc Range[requestPath.Count('/') + 1];
        requestPath.Split(ranges, '/');
        
        return ranges.Length >= 3
            ? requestPath[ranges[3]].ToString()
            : string.Empty;
    }
    
    
    private static async Task<Post> FetchPost(string postId, HttpClient client)
    {
        var response = await client.GetAsync($"http://media-service:8080/api/posts/{postId}");
        return (await response.Content.ReadFromJsonAsync<Post>())!;
    }

    private static async Task<PostInteractionResponse> FetchRepliesForPost(string postId,string? userId, HttpClient client)
    {
        string requestUri = string.IsNullOrEmpty(userId)
            ? $"http://user-interaction:8080/api/posts/{postId}/replies"
            : $"http://user-interaction:8080/api/posts/{postId}/replies?userId={userId}";
        
        var response = await client.GetAsync(requestUri);
        return (await response.Content.ReadFromJsonAsync<PostInteractionResponse>())!;
    }
}