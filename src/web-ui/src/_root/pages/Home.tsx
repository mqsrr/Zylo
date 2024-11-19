import {usePostContext} from "@/hooks/usePostContext.ts";
import {useEffect} from "react";
import PostCard from "@/components/shared/PostCard.tsx";

const Home = () => {
    const {feed, isLoading: isFeedLoading, fetchNextPage} = usePostContext();
    useEffect(() => {
        const handleScroll = () => {
            const scrollHeight = document.documentElement.scrollHeight;
            const scrollTop = document.documentElement.scrollTop;
            const clientHeight = document.documentElement.clientHeight;

            if (scrollHeight - scrollTop <= clientHeight + 100) {
                fetchNextPage().catch(console.error);
            }
        };

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
                    {isFeedLoading && !feed ? (
                        <div>Loading</div>
                    ): (
                        <ul className="flex flex-col flex-1 gap-9 w-full">
                            {feed.map((post) => (
                                <PostCard post={post} key={post.id}/>
                            ))}
                        </ul>
                    )}
                </div>
            </div>

        </div>
    );
};

export default Home;