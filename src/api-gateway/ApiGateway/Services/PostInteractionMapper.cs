using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using ApiGateway.Models;

namespace ApiGateway.Services;

internal static class PostInteractionMapper
{
    private static ref Post SearchById(Span<Post> posts, Ulid id)
    {
        for (int i = 0; i < posts.Length; i++)
        {
            ref var current = ref posts[i];
            if (current.Id == id)
            {
                return ref current;
            }
        }

        return ref Unsafe.NullRef<Post>();
    }

    public static List<Post> MapPostInteractions<T, TR>(T posts, TR replies)
        where T : ICollection<Post>
        where TR : ICollection<PostInteractionResponse>
    {
        
        Span<Post> postSpan = ConvertToSpan(posts);
        Span<PostInteractionResponse> repliesSpan = ConvertToSpan(replies);

        postSpan.Sort((x, y) => x.Id.CompareTo(y.Id));
        foreach (var reply in repliesSpan)
        {
            ref var post = ref SearchById(postSpan, reply.PostId);
            if (Unsafe.IsNullRef(ref post))
            {
                continue;
            }

            post.Replies = reply.Replies;
            post.Likes = reply.Likes;
            post.Views = reply.Views;
            post.UserInteracted = reply.UserInteracted;
        }

        var result = new List<Post>(postSpan.Length);
        result.AddRange(posts);

        return result;
    }
    
    private static Span<T> ConvertToSpan<T>(ICollection<T> collection)
    {
        return collection switch
        {
            T[] array => array.AsSpan(),
            List<T> list => CollectionsMarshal.AsSpan(list),
            _ => throw new InvalidOperationException("Unsupported collection type")
        };
    }
    
}