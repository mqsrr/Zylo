import {usePostContext} from "@/hooks/usePostContext.ts";
import {useCallback, useEffect, useState} from "react";
import PostCard from "@/components/shared/PostCard.tsx";
import PostService from "@/services/PostService.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {debounce} from "@/lib/utils.ts";


const Home = () => {
    const { addOrUpdatePost } = usePostContext();
    const { userId, accessToken } = useAuthContext();

    const [feedPostIds, setFeedPostIds] = useState<string[]>([]);
    const [next, setNext] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    useEffect(() => {
        const fetchInitialPosts = async () => {
            if (!userId || !accessToken) return;

            setIsLoading(true);

            try {
                const response = await PostService.getUsersFeed(userId, accessToken.value);
                if (response) {
                    setFeedPostIds(response.data.map((post) => post.id));
                    setNext(response.next || null);
                    response.data.forEach(addOrUpdatePost);
                }
            } catch (error) {
                console.error("Error fetching initial posts:", error);
            } finally {
                setIsLoading(false);
            }
        };

        fetchInitialPosts().catch(console.error);
    }, [userId, accessToken, addOrUpdatePost]);

    const fetchNextPage = useCallback(async () => {
        if (!next || isLoading || !userId || !accessToken) return;

        setIsLoading(true);

        try {
            const response = await PostService.getUsersFeed(userId, accessToken.value, next);
            if (response) {
                setFeedPostIds((prevIds) => [...prevIds, ...response.data.map((post) => post.id)]);
                setNext(response.next || null);
                response.data.forEach(addOrUpdatePost);
            }
        } catch (error) {
            console.error("Error fetching next page:", error);
        } finally {
            setIsLoading(false);
        }
    }, [next, isLoading, userId, accessToken, addOrUpdatePost]);

    useEffect(() => {
        const handleScroll = debounce(() => {
            const scrollHeight = document.documentElement.scrollHeight;
            const scrollTop = document.documentElement.scrollTop;
            const clientHeight = document.documentElement.clientHeight;

            if (scrollHeight - scrollTop <= clientHeight + 100) {
                fetchNextPage().catch(console.error);
            }
        }, 300); // Adjust debounce delay as needed

        window.addEventListener("scroll", handleScroll);

        return () => {
            window.removeEventListener("scroll", handleScroll);
        };
    }, [fetchNextPage]);


    return (
        <div className="flex flex-1">
            <div className="home-container">
                <div className="home-posts">
                    <h2 className="h3-bold md:h2-bold text-left w-full"> Home feed</h2>
                    {isLoading && feedPostIds.length === 0 ? (
                        <div>Loading</div>
                    ): (
                        <ul className="flex flex-col flex-1 gap-9 w-full">
                            {feedPostIds.map((id) => (
                                <PostCard postId={id} key={id}/>
                            ))}
                        </ul>
                    )}
                    {isLoading && feedPostIds.length > 0 && <div>Loading more posts...</div>}
                </div>
            </div>

        </div>
    );
};

export default Home;