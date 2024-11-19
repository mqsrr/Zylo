import {
    CreatePostUri, CreateReplyUri,
    DeletePostUri, DeleteReplyUri,
    DownloadPostMediaUri,
    GetPostUri, GetUsersFeed, LikePostUri, UnlikePostUri,
    UpdatePostUri, UpdateReplyContentUri, ViewPostUri
} from "@/constants/requestsUri.ts";
import {Post} from "@/models/Post.ts";
import {PaginatedResponse} from "@/models/PaginatedResponse.ts";
import {Reply} from "@/models/Reply.ts";

class PostService {

    getPost = async (id: string, userId:string, token: string): Promise<Post | null> => {
        const response = await fetch(GetPostUri(id, userId), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
            ? await response.json()
            : null;
    }


    getUsersFeed = async (token: string, userId: string, next?: string, perPage?: string): Promise<PaginatedResponse<Post> | null> => {
        const response = await fetch(GetUsersFeed(userId, next, perPage), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
            ? await response.json()
            : null;
    }

    createPost = async (id: string, request: FormData, token: string): Promise<Post | null> => {
        const response = await fetch(CreatePostUri(id), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
            body: request,
            method: "POST"
        });

        return response.ok
            ? await response.json()
            : null;
    }

    likePost = async (userId: string, postId: string, token: string): Promise<boolean> => {
        const response = await fetch(LikePostUri(userId, postId), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
            method: "POST"
        });

        return response.status == 201;
    }

    unlikePost = async (userId: string, postId: string, token: string): Promise<boolean> => {
        const response = await fetch(UnlikePostUri(userId, postId), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
            method: "DELETE"
        });

        return response.status == 204;
    }

    viewPost = async (userId: string, postId: string, token: string): Promise<boolean> => {
        const response = await fetch(ViewPostUri(userId, postId), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
            method: "POST"
        });

        return response.status == 201;
    }

    downloadMedia = async (postId: string, mediaId: string, token: string): Promise<Blob> => {
        const response = await fetch(DownloadPostMediaUri(postId, mediaId), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.message || 'Error making request');
        }

        return await response.blob();
    }

    updatePost = async (userId: string, postId: string, formData: FormData, token: string): Promise<Post | null> => {
        const response = await fetch(UpdatePostUri(userId, postId), {
            method: 'PUT',
            headers: {
                Authorization: `Bearer ${token}`,
            },
            body: formData,
        });

        return response.ok
            ? await response.json()
            : null;
    }

    deletePost = async (userId: string, postId: string, token: string): Promise<boolean> => {
        const response = await fetch(DeletePostUri(userId, postId), {
            method: 'DELETE',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
    }

    createReply = async (userId: string, repliedPostId: string, content: string ,token: string): Promise<Reply | null> => {
        const response = await fetch(CreateReplyUri(repliedPostId), {
            method: 'POST',
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                "userId": userId,
                "replyToId": repliedPostId,
                "content": content
            })
        });

        return response.ok
            ? await response.json()
            : null;
    }

    updateReply = async (repliedPostId: string, replyId: string, content: string, token: string): Promise<Reply | null> => {
        const response = await fetch(UpdateReplyContentUri(repliedPostId, replyId), {
            method: 'PUT',
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                "content": content
            })
        });

        return response.ok
            ? await response.json()
            : null;
    }

    deleteReply = async (repliedPostId: string ,replyId: string, token: string): Promise<boolean> => {
        const response = await fetch(DeleteReplyUri(repliedPostId, replyId), {
            method: 'DELETE',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
    }
}

export default new PostService()