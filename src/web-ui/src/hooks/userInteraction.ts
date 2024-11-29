import { useState } from "react";
import { useAuthContext } from "@/hooks/useAuthContext.ts";
import { usePostContext } from "@/hooks/usePostContext.ts";
import PostService from "@/services/PostService.ts";
import {Reply} from "@/models/Reply.ts";
import {Post} from "@/models/Post.ts";

type InteractionTarget = {
    id: string;
    likes: number;
    userInteracted: boolean;
    replyToId?: string; // Optional for replies
};

export const useInteraction = (target: InteractionTarget, isReply: boolean) => {
    const { userId, accessToken } = useAuthContext();
    const { posts, addOrUpdatePost } = usePostContext();
    const [likes, setLikes] = useState(target.likes);
    const [isLiked, setIsLiked] = useState(target.userInteracted);

    const handleLike = async () => {
        if (!userId || !accessToken) return;

        setLikes(isLiked ? likes - 1 : likes + 1);
        const action = isLiked ? PostService.unlikePost : PostService.likePost;

        setIsLiked(!isLiked);
        const isUpdated = await action(userId, target.id, accessToken.value);
        if (!isUpdated) {
            setLikes(isLiked ? likes - 1 : likes + 1);
            setIsLiked(!isLiked);
        }

        const updatedTarget = {
            ...target,
            likes,
            userInteracted: !target.userInteracted,
        };

        if (isReply && target.replyToId) {
            const parentPost = posts[target.replyToId];
            if (parentPost) {
                const updatedReplies = updateNestedReply(parentPost.replies!, target.id, updatedTarget as Reply);
                addOrUpdatePost({ ...parentPost, replies: updatedReplies });
            }
        } else {
            addOrUpdatePost(updatedTarget as Post);
        }
    };

    return { likes, isLiked, handleLike };
};

const updateNestedReply = (
    replies: Reply[],
    targetId: string,
    updatedReply: Reply
): Reply[] => {
    return replies.map((reply) => {
        if (reply.id === targetId) {
            return { ...reply, ...updatedReply };
        }
        if (reply.nestedReplies && reply.nestedReplies.length > 0) {
            return { ...reply, replies: updateNestedReply(reply.nestedReplies, targetId, updatedReply) };
        }
        return reply;
    });
};
