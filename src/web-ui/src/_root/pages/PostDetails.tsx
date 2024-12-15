import {Link, useParams} from "react-router-dom";
import {useEffect, useState} from "react";
import {Post} from "@/models/Post.ts";
import {Card, CardContent} from "@/components/ui/card.tsx";
import ReplyItem from "@/components/shared/ReplyItem.tsx";
import {Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious} from "@/components/ui/carousel.tsx";
import PostInteractions from "@/components/shared/PostInteractions.tsx";
import {formatDistanceToNow} from "date-fns";
import PostService from "@/services/PostService.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {Reply} from "@/models/Reply.ts";
import {usePostContext} from "@/hooks/usePostContext.ts";

const PostDetails = () => {
    const { id } = useParams<{ id: string }>();
    const {getPostById, fetchPostById} = usePostContext();
    const { userId, accessToken } = useAuthContext();
    const [post, setPost] = useState<Post | null>(null);
    const [replies, setReplies] = useState<Reply[]>([]);


    useEffect(() => {
        if (!id || !userId || !accessToken) {
            return;
        }

        const initializePost = async (): Promise<void> => {
            let post = getPostById(id)
            if (!post) {
                post = await fetchPostById(id);
                if (!post) return;
            }

            setPost(post);
            setReplies(post.replies || []);

            const isUpdated = await PostService.viewPost(userId, id, accessToken.value);
            if (!isUpdated) {
                return;
            }

            setPost(prevPost => prevPost ? {...prevPost, views: prevPost.views + 1} : prevPost)
        }

        initializePost().catch(console.error)
    }, [id, userId, accessToken, getPostById, fetchPostById]);

    const handleReplySubmit = (newReply: Reply) => {
        setReplies([newReply, ...replies]);
    };

    if (post === null) {
        return <div>Loading</div>;
    }

    return (
        <div className="container mx-auto px-4 py-6 overflow-auto">
            <Card className="shadow-md mb-6">
                <CardContent>
                    <div className="flex items-center mb-4 mt-4">
                        <Link
                            to={`/profile/${post.user.id}`}
                            onClick={(e) => e.stopPropagation()}
                        >
                            <img
                                src={post.user.profileImage.url}
                                alt={post.user.profileImage.fileName}
                                className="rounded-full w-12 h-12 object-cover"
                            />
                        </Link>
                        <div className="ml-3">
                            <p className="font-semibold text-lg">{post.user.name}</p>
                            <p className="text-sm text-gray-500">
                                {formatDistanceToNow(new Date(post.createdAt), {
                                    addSuffix: true,
                                })}
                            </p>
                        </div>
                    </div>

                    <p className="mb-4">{post.text}</p>

                    {post.filesMetadata && post.filesMetadata.length > 0 && (
                        <div className="flex justify-center mb-4">
                            <div className="w-full max-w-2xl relative">
                                <Carousel className="w-full">
                                    <CarouselContent>
                                        {post.filesMetadata.map((file) => (
                                            <CarouselItem key={file.fileName}>
                                                <img
                                                    src={file.url}
                                                    alt={file.fileName}
                                                    className="w-full h-auto object-contain rounded-2xl"
                                                />
                                            </CarouselItem>
                                        ))}
                                    </CarouselContent>
                                    {post.filesMetadata.length > 1 && (
                                        <>
                                            <CarouselPrevious />
                                            <CarouselNext />
                                        </>
                                    )}
                                </Carousel>
                            </div>
                        </div>
                    )}

                    <PostInteractions
                        post={post}
                        isTopLevel={true}
                        onReplySubmit={handleReplySubmit}
                    />
                </CardContent>
            </Card>

            {replies.length > 0 && (
                <div>
                    {replies.map((reply) => (
                        <Card key={reply.id} className="mb-4">
                            <CardContent className="mt-4">
                                <ReplyItem
                                    reply={reply}
                                    postId={post.id}
                                    level={0}
                                    maxExpandLevel={3}
                                />
                            </CardContent>
                        </Card>
                    ))}
                </div>
            )}
        </div>
    );
};
export default PostDetails;