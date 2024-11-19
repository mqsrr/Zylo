using System.Net;
using System.Text;
using ApiGateway.Models;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

namespace ApiGateway.DelegatingHandlers;

public sealed class PostDelegatingHandler : DelegatingHandler
{
    private readonly IHttpClientFactory _clientFactory;
    
    public PostDelegatingHandler(IHttpClientFactory clientFactory)
    {
        _clientFactory = clientFactory;
    }
    
    protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
    {
        var downstreamResponse = await base.SendAsync(request, cancellationToken);
        if (!downstreamResponse.IsSuccessStatusCode)
        {
            return downstreamResponse;
        }
        
        var post = await downstreamResponse.Content.ReadFromJsonAsync<Post>(cancellationToken);
        if (post is null)
        {
            return new HttpResponseMessage
            {
                RequestMessage = request,
                StatusCode = HttpStatusCode.NotFound,
                ReasonPhrase = "Post not found"
            };
        }

        string userId = string.Empty;
        string query = request.RequestUri!.Query;
        if (query.Contains("userId"))
        {
            userId = query[8..];
        }

        Console.WriteLine(userId);
        string authorizeHeader = request.Headers.Authorization!.ToString();
        
        var httpClient = _clientFactory.CreateClient();
        httpClient.DefaultRequestHeaders.Add("Authorization", authorizeHeader);

        var userInteractions = await FetchRepliesForPost(post.Id.ToString(), userId, httpClient);
        if (userInteractions.Replies.Any())
        {
            post.Replies = userInteractions.Replies;
        }
        
        post.UserInteracted = userInteractions.UserInteracted;
        post.Likes = userInteractions.Likes;
        post.Views = userInteractions.Views;
        
        string aggregatedContent = JsonConvert.SerializeObject(post, new JsonSerializerSettings
        {
            NullValueHandling = NullValueHandling.Ignore,
            ContractResolver = new CamelCasePropertyNamesContractResolver()
        });
        
        var stringContent = new StringContent(aggregatedContent, Encoding.UTF8, "application/json");
        var httpResponse = new HttpResponseMessage
        {
            Content = stringContent,
            ReasonPhrase = "OK",
            RequestMessage = request,
            StatusCode = HttpStatusCode.OK,
        };
        
        return httpResponse;
    }
    
    private static async Task<PostInteractionResponse> FetchRepliesForPost(string postId,string userId, HttpClient client)
    {
        string requestUri = string.IsNullOrEmpty(userId)
            ? $"http://localhost:8083/api/posts/{postId}/replies"
            : $"http://localhost:8083/api/posts/{postId}/replies?userId={userId}";
        
        var response = await client.GetAsync(requestUri);
        return (await response.Content.ReadFromJsonAsync<PostInteractionResponse>())!;
    }
}