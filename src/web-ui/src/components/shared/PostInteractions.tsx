import {Post} from "@/models/Post.ts";
import {EyeIcon, HeartIcon} from "lucide-react";
import PostService from "@/services/PostService.ts";
import React, {useState} from "react";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {Reply} from "@/models/Reply.ts";
import {Button} from "@/components/ui/button.tsx";
import {Input} from "@/components/ui/input.tsx";

type PostInteractionsProps = {
    post: Post | Reply;
    isTopLevel: boolean;
    onReplySubmit?: (reply: Reply) => void;
};

const PostInteractions = ({post, isTopLevel = true, onReplySubmit}: PostInteractionsProps) => {
    const {userId, accessToken} = useAuthContext();

    const [likes, setLikes] = useState(post.likes);
    const [isLiked, setIsLiked] = useState(post.userInteracted);
    const [views] = useState(post.views);

    const [showReplyInput, setShowReplyInput] = useState(false);
    const [replyContent, setReplyContent] = useState("");

    const handleLikePost = async (e: React.MouseEvent) => {
        e.stopPropagation();

        if (!userId || !accessToken) return;

        setLikes(isLiked ? likes - 1 : likes + 1);
        const action = !isLiked ? PostService.likePost : PostService.unlikePost;

        setIsLiked(!isLiked);
        const isUpdated = await action(userId, post.id, accessToken.value);
        if (!isUpdated) {
            setLikes(isLiked ? likes - 1 : likes + 1);
            setIsLiked(!isLiked);
        }
    };

    const toggleReplyInput = (e: React.MouseEvent) => {
        e.stopPropagation();
        setShowReplyInput(!showReplyInput);
    };

    const handleReplySubmit = async () => {
        if (!replyContent.trim() || !userId || !accessToken) return;

        try {

            const reply = await PostService.createReply(userId, post.id, replyContent, accessToken.value);
            if (!reply) {
                return;
            }

            setReplyContent("");
            setShowReplyInput(false);

            if (onReplySubmit) {
                onReplySubmit(reply);
            }

        } catch (error) {
            console.error("Error submitting reply:", error);
        }
    };

    return (
        <div
            className={`flex flex-col ${
                isTopLevel ? "items-start" : "items-start ml-4"
            }`}>
            <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                    <HeartIcon
                        size={isTopLevel ? 20 : 16}
                        color="#ff4d4f"
                        fill={isLiked ? "#ff4d4f" : "none"}
                        onClick={handleLikePost}
                        className="cursor-pointer"
                    />
                    <p className={isTopLevel ? "small-medium lg:base-medium" : "text-sm"}>
                        {likes}
                    </p>
                </div>
                <div className="flex items-center gap-2">
                    <EyeIcon size={isTopLevel ? 20 : 16} color="#877eff"/>
                    <p className={isTopLevel ? "small-medium lg:base-medium" : "text-sm"}>
                        {views}
                    </p>
                </div>
                <Button onClick={toggleReplyInput} variant="link" size="icon">
                    {showReplyInput ? "Cancel" : "Reply"}
                </Button>
            </div>

            {showReplyInput && (
                <div className="mt-2 w-full">
                    <Input
                        value={replyContent}
                        onChange={(e) => setReplyContent(e.target.value)}
                        placeholder="Write your reply..."/>
                    <Button onClick={handleReplySubmit} className="mt-4 px-4" size="sm">
                        Submit Reply
                    </Button>
                </div>
            )}
        </div>
    );
};

export default PostInteractions;