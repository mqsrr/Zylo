import React, {createContext, ReactNode, useCallback, useState,} from "react";
import {Post} from "@/models/Post.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import PostService from "@/services/PostService.ts";

interface PostContextType {
    posts: Post[];
    getPostById: (postId: string) => Post | null;
    addOrUpdatePost: (post: Post) => void;
}

export const PostContext = createContext<PostContextType | undefined>(undefined);

export const PostProvider: React.FC<{ children: ReactNode }> = ({children}) => {
    const [posts, setPosts] = useState<Post[]>([]);
    const {userId, accessToken} = useAuthContext();

    const getPostById = useCallback((postId: string) => {

        const post = posts.find((post) => post.id === postId);
        if (!post && userId && accessToken) {
            PostService.getPost(postId, userId, accessToken.value)
                .then(post => {
                    if (post) {
                        posts.push(post);
                        setPosts(posts);
                    }

                    return post;
                });
        }

        return post ? post : null
    }, [posts, userId, accessToken]);

    const addOrUpdatePost = useCallback((post: Post) => {
        setPosts((prevPosts) => prevPosts.map(p => {
            if (p.id === post.id) {
                return post;
            }
            return p;
        }));
    }, []);

    return (
        <PostContext.Provider
            value={{
                posts,
                getPostById,
                addOrUpdatePost,
            }}>
            {children}
        </PostContext.Provider>
    );
};

