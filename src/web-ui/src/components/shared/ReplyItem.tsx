import {useState} from "react";
import {Link, useNavigate} from "react-router-dom";
import {Reply} from "@/models/Reply.ts";
import PostInteractions from "@/components/shared/PostInteractions.tsx";
import {formatDistanceToNow} from "date-fns";
import PostService from "@/services/PostService.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {Button} from "@/components/ui/button.tsx";
import {Input} from "@/components/ui/input.tsx";

type ReplyItemProps = {
    reply: Reply;
    postId: string;
    level?: number;
    maxExpandLevel?: number;
};

const ReplyItem = ({reply, postId, level = 0, maxExpandLevel = 1}: ReplyItemProps) => {
    const {userId, accessToken} = useAuthContext();

    const [showNestedReplies, setShowNestedReplies] = useState(false);
    const [nestedReplies, setNestedReplies] = useState(reply.nestedReplies || []);
    const [isEditing, setIsEditing] = useState(false);
    const [editContent, setEditContent] = useState(reply.content);

    const relativeDate = formatDistanceToNow(new Date(reply.createdAt), {addSuffix: true});
    const navigate = useNavigate();

    const canShowNestedReplies = level + 1 < maxExpandLevel;

    const handleNavigateToReplyDetails = () => {
        navigate(`/posts/${postId}/replies/${reply.id}`);
    };

    const handleReplySubmit = (newReply: Reply) => {
        setNestedReplies([newReply, ...nestedReplies]);
        setShowNestedReplies(true);
    };

    const handleEditClick = () => {
        setIsEditing(true);
    };

    const handleEditCancel = () => {
        setIsEditing(false);
        setEditContent(reply.content);
    };

    const handleEditSave = async () => {
        if (!editContent.trim() || !accessToken || !userId) {
            return;
        }

        try {
            const updatedReply = await PostService.updateReply(reply.replyToId, reply.id, editContent, accessToken.value);
            if (!updatedReply) {
                console.error("Could not update reply");
                return;
            }

            reply.content = updatedReply.content;
            setIsEditing(false);
        } catch (error) {
            console.error("Error updating reply:", error);
        }
    };


    return (
        <div style={{paddingLeft: `${level * 1.5}rem`}} className="mb-4 border-l pl-4">
            <div className="ml-4">
                <div className="flex items-center mb-2">
                    <Link to={`/profile/${reply.user.id}`} onClick={(e) => e.stopPropagation()}>
                        <img
                            src={reply.user.profileImage.url}
                            alt={reply.user.profileImage.fileName}
                            className="rounded-full w-8 h-8 object-cover"/>
                    </Link>
                    <div className="ml-2">
                        <p className="text-sm font-semibold">{reply.user.name}</p>
                        <p className="text-xs text-gray-500">{relativeDate}</p>
                    </div>
                    {reply.user.id === userId && !isEditing && (
                        <Button onClick={handleEditClick} className="ml-auto" size="icon" variant="link">
                            Edit
                        </Button>
                    )}
                </div>

                {!isEditing ? (
                    <p className="text-sm mb-2 cursor-pointer" onClick={handleNavigateToReplyDetails}>
                        {reply.content}
                    </p>
                ) : (
                    <div className="mt-2">
                        <Input value={editContent} onChange={(e) => setEditContent(e.target.value)}
                               className="w-full p-2"/>
                        <div className="flex space-x-2 mt-2">
                            <Button onClick={handleEditSave} className="px-6" size="sm">
                                Save
                            </Button>
                            <Button onClick={handleEditCancel} className="px-6" size="sm" variant="secondary">
                                Cancel
                            </Button>
                        </div>
                    </div>
                )}

                <PostInteractions
                    post={reply}
                    isTopLevel={false}
                    onReplySubmit={handleReplySubmit}/>

                {nestedReplies.length > 0 && canShowNestedReplies && (
                    <div>
                        {!showNestedReplies && (
                            <p className="text-sm text-gray-600 cursor-pointer mb-2" onClick={() => setShowNestedReplies(true)}>
                                View replies ({nestedReplies.length})
                            </p>
                        )}
                        {showNestedReplies && (
                            <div className="mt-2">
                                {nestedReplies.map((nestedReply) => (
                                    <ReplyItem
                                        key={nestedReply.id}
                                        reply={nestedReply}
                                        postId={postId}
                                        level={level + 1}
                                        maxExpandLevel={maxExpandLevel}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
};

export default ReplyItem;