import React, {createContext, ReactNode, useCallback, useEffect, useState,} from "react";
import {Post} from "@/models/Post.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import PostService from "@/services/PostService.ts";
import {PaginatedResponse} from "@/models/PaginatedResponse.ts";

interface PostsContextType {
    isLoading: boolean;
    feed: Post[];
    updatePostInFeed: (updatedPost: Post) => void;
    fetchNextPage: () => Promise<void>;
}

export const PostContext = createContext<PostsContextType | undefined>(undefined);
export const PostsProvider: React.FC<{ children: ReactNode }> = ({children}) => {
    const {accessToken, userId} = useAuthContext();
    const [isLoading, setIsLoading] = useState(false);
    const [feed, setFeed] = useState<Post[]>([]);
    const [feedPage, setFeedPage] = useState<PaginatedResponse<Post> | null>(null)

    const updatePostInFeed = (updatedPost: Post) => {
        setFeed((prevFeed) =>
            prevFeed.map((post) => (post.id === updatedPost.id ? updatedPost : post))
        );
    }

    const fetchNextPage = useCallback(async () => {
        if (!accessToken || !userId || feedPage !== null && !feedPage.hasNextPage) {
            return;
        }
        setIsLoading(true);

        const paginatedResponse = await PostService.getUsersFeed(accessToken.value, userId, feedPage?.next, "10");
        console.log(paginatedResponse);
        if (paginatedResponse === null || paginatedResponse.data.length === 0){
            setIsLoading(false);
            return
        }

        setFeedPage(paginatedResponse)
        setFeed((prevPosts) => [...prevPosts, ...paginatedResponse.data]);

        setIsLoading(false);
    }, [accessToken, userId, feedPage, setFeedPage]);


    useEffect(() => {
        console.log("fetching posts")
        fetchNextPage().catch((error) => console.error(error));

    }, [accessToken, userId, fetchNextPage])


    return (
        <PostContext.Provider value={{
            fetchNextPage,
            isLoading,
            feed,
            updatePostInFeed
        }}>
            {children}
        </PostContext.Provider>
    );
};

