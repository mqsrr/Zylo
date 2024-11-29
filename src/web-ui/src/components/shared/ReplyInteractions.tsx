import { useState} from "react";
import { EyeIcon, HeartIcon } from "lucide-react";
import { Reply } from "@/models/Reply.ts";
import { Button } from "@/components/ui/button.tsx";
import { Input } from "@/components/ui/input.tsx";
import PostService from "@/services/PostService.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {useInteraction} from "@/hooks/userInteraction.ts";

type ReplyInteractionProps = {
    reply: Reply;
    onReplySubmit: (reply: Reply) => void;
};

const ReplyInteraction = ({ reply, onReplySubmit }: ReplyInteractionProps) => {
    const { likes, isLiked, handleLike } = useInteraction(reply, true);
    const [showReplyInput, setShowReplyInput] = useState(false);
    const {userId, accessToken} = useAuthContext();
    const [replyContent, setReplyContent] = useState("");

    const toggleReplyInput = () => {
        setShowReplyInput(!showReplyInput);
    };

    const handleSubmitReply = async () => {
        if (!replyContent.trim() || !userId || !accessToken) return;

        try {
            const newReply = await PostService.createReply(userId, reply.id, replyContent, accessToken.value);
            if (newReply) {
                onReplySubmit(newReply);
                setReplyContent("");
                setShowReplyInput(false);
            }
        } catch (error) {
            console.error("Error submitting reply:", error);
        }
    };

    return (
        <div className="flex flex-col items-start ml-4">
            <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                    <HeartIcon
                        size={16}
                        color="#ff4d4f"
                        fill={isLiked ? "#ff4d4f" : "none"}
                        onClick={handleLike}
                        className="cursor-pointer"
                    />
                    <p className="text-sm">{likes}</p>
                </div>
                <div className="flex items-center gap-2">
                    <EyeIcon size={16} color="#877eff" />
                    <p className="text-sm">{reply.views}</p>
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
                        placeholder="Write your reply..."
                    />
                    <Button onClick={handleSubmitReply} className="mt-4 px-4" size="sm">
                        Submit Reply
                    </Button>
                </div>
            )}
        </div>
    );
};

export default ReplyInteraction;
