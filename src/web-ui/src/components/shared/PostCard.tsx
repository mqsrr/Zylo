import {Post} from "@/models/Post.ts";
import {Link, useNavigate} from "react-router-dom";
import {
    Carousel,
    CarouselContent,
    CarouselItem,
} from "@/components/ui/carousel.tsx";
import PostInteractions from "./PostInteractions";
import {Card, CardContent} from "@/components/ui/card.tsx";
import Replies from "./Replies";
import {useUserContext} from "@/hooks/useTokenContext.ts";
import {EditIcon} from "lucide-react";
import React, {useState} from "react";
import {formatDistanceToNow} from "date-fns";
import {Reply} from "@/models/Reply.ts";

type PostCardProps = {
    post: Post;
};

const PostCard = ({post}: PostCardProps) => {
    const {user} = useUserContext();
    const [replies, setReplies] = useState<Reply[]>(post.replies || []);
    const relativeDate = formatDistanceToNow(new Date(post.createdAt), {
        addSuffix: true,
    });
    const navigate = useNavigate();

    const onEditClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        navigate(`/edit/posts/${post.id}`);
    };

    const handleReplySubmit = (newReply: Reply) => {
        setReplies([newReply, ...replies]);
    };

    return (
        <Card className="shadow-md rounded-lg overflow-hidden mb-6">
            <CardContent>
                <div className="flex items-center mb-4 mt-4">
                    <Link to={`/profile/${post.user.id}`} onClick={(e) => e.stopPropagation()}>
                        <img
                            src={post.user.profileImage.url}
                            alt={post.user.profileImage.fileName}
                            className="rounded-full w-12 h-12 object-cover"
                        />
                    </Link>

                    <div className="ml-3">
                        <p className="font-semibold text-lg">{post.user.name}</p>
                        <p className="text-sm text-gray-500">{relativeDate}</p>
                    </div>
                    {post.user.id === user?.id && (
                        <EditIcon
                            className="ml-auto mr-0 cursor-pointer"
                            size={20}
                            onClick={onEditClick}
                        />
                    )}
                </div>
                <p className="mb-4">{post.text}</p>
                {post.filesMetadata && post.filesMetadata.length > 0 && (
                    <Link to={`/posts/${post.id}`}>
                        <Carousel className="w-full mb-4">
                            <CarouselContent>
                                {post.filesMetadata.map((file) => (
                                    <CarouselItem key={file.fileName}>
                                        <img
                                            src={file.url}
                                            alt={file.fileName}
                                            className="object-cover w-full h-64 rounded-2xl"
                                        />
                                    </CarouselItem>
                                ))}
                            </CarouselContent>
                        </Carousel>
                    </Link>
                )}

                <PostInteractions
                    post={post}
                    isTopLevel={true}
                    onReplySubmit={handleReplySubmit}
                />

                {replies.length > 0 && (
                    <Replies replies={replies} postId={post.id} maxExpandLevel={2}/>
                )}
            </CardContent>
        </Card>
    );
};

export default PostCard;