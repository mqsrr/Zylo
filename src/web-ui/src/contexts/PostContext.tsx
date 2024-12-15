import React, {createContext, ReactNode, useCallback, useState,} from "react";
import {Post} from "@/models/Post.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import PostService from "@/services/PostService.ts";

interface PostContextType {
    posts: Post[];
    getPostById: (postId: string) => Post | null;
    addOrUpdatePost: (post: Post) => void;
    fetchPostById: (postId: string) => Promise<Post | null>;
    findParentPostByReplyId: (replyId: string) => Post | null;
}

export const PostContext = createContext<PostContextType | undefined>(undefined);

export const PostProvider: React.FC<{ children: ReactNode }> = ({children}) => {
    const [posts, setPosts] = useState<Post[]>([]);
    const {userId, accessToken} = useAuthContext();

    const getPostById = useCallback((postId: string) => {
        const post = posts.find((p) => p.id === postId);
        return post || null;
    }, [posts]);


    const addOrUpdatePost = useCallback((post: Post) => {
        setPosts((prevPosts) => {
            const index = prevPosts.findIndex((p) => p.id === post.id);
            if (index !== -1) {
                const updatedPosts = [...prevPosts];
                updatedPosts[index] = post;
                return updatedPosts;
            } else {
                return [...prevPosts, post];
            }
        });
    }, []);

    const fetchPostById = useCallback(
        async (postId: string) => {
            const existingPost = getPostById(postId);
            if (existingPost) {
                return existingPost;
            }
            if (!userId || !accessToken) {
                return null;
            }

            try {
                const fetchedPost = await PostService.getPost(postId, userId, accessToken.value);
                if (fetchedPost) {
                    addOrUpdatePost(fetchedPost);
                    return fetchedPost;
                }
            } catch (error) {
                console.error("Error fetching post by ID:", error);
            }
            return null;
        },
        [userId, accessToken, addOrUpdatePost, getPostById]
    );

    const findReplyInTree = (replies: Post["replies"], replyId: string): boolean => {
        if (!replies) return false;
        for (const reply of replies) {
            if (reply.id === replyId) {
                return true; // Found the reply
            }
            // If this reply has nested replies (treated as Post for convenience), search deeper
            if (reply.nestedReplies && findReplyInTree(reply.nestedReplies, replyId)) {
                return true;
            }
        }
        return false;
    };

    const findParentPostByReplyId = useCallback((replyId: string): Post | null => {
        for (const post of posts) {
            if (findReplyInTree(post.replies, replyId)) {
                return post;
            }
        }
        return null;
    }, [posts]);

    return (
        <PostContext.Provider
            value={{
                posts,
                getPostById,
                fetchPostById,
                addOrUpdatePost,
                findParentPostByReplyId
            }}>
            {children}
        </PostContext.Provider>
    );
};

