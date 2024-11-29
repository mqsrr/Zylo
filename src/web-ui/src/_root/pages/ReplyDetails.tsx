import {Link, useParams} from "react-router-dom";
import React, {useEffect, useState} from "react";
import {Reply} from "@/models/Reply.ts";
import {Card, CardContent, CardHeader} from "../../components/ui/card.tsx";
import ReplyItem from "@/components/shared/ReplyItem.tsx";
import {formatDistanceToNow} from "date-fns";
import PostInteractions from "@/components/shared/PostInteractions.tsx";
import PostService from "@/services/PostService.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {Button} from "@/components/ui/button.tsx";
import {Input} from "@/components/ui/input.tsx";

const ReplyDetail = () => {
        const {postId, replyId} = useParams<{ postId: string; replyId: string }>();
        const {userId, accessToken} = useAuthContext();
        const [reply, setReply] = useState<Reply | null>(null);

        useEffect(() => {
            if (!replyId || !postId || !userId || !accessToken) {
                return;
            }

            const initializeReply = async () => {
                const reply =await PostService.getReply(replyId, postId, userId, accessToken.value);
                if (!reply) {
                    console.error("Could not find reply!");
                    return;
                }

                setReply(reply)

                const isUpdated = await PostService.viewPost(userId, replyId, accessToken.value);
                if (!isUpdated) {
                    return;
                }
                console.log(reply)
                setReply({...reply, views: reply.views + 1});
            }


            initializeReply().catch(console.error);
        }, [replyId, postId, userId, accessToken]);


        if (reply === null || postId === undefined) {
            console.log(reply)
            return <div>Loading</div>;
        }

        return (
            <div className="container mx-auto px-4 py-6 overflow-auto">
                <Card className="mb-6">
                    <CardHeader>
                        <h2 className="text-xl font-semibold">Reply Details</h2>
                    </CardHeader>
                    <CardContent>
                        <MainReplyCard reply={reply} setReply={setReply}/>
                    </CardContent>
                </Card>

                {reply.nestedReplies && reply.nestedReplies.length > 0 && (
                    <div>
                        {reply.nestedReplies.map((nestedReply) => (
                            <Card key={nestedReply.id} className="mb-4">
                                <CardContent className="mt-4">
                                    <ReplyItem
                                        reply={nestedReply}
                                        postId={postId}
                                        level={1}
                                        maxExpandLevel={3}
                                    />
                                </CardContent>
                            </Card>
                        ))}
                    </div>
                )}
            </div>
        );
    }
;

export default ReplyDetail;

type MainReplyCardProps = {
    reply: Reply;
    setReply: React.Dispatch<React.SetStateAction<Reply | null>>;
};

const MainReplyCard = ({reply, setReply}: MainReplyCardProps) => {
    const {userId, accessToken} = useAuthContext();
    const relativeDate = formatDistanceToNow(new Date(reply.createdAt), {
        addSuffix: true,
    });
    const [isEditing, setIsEditing] = useState(false);
    const [editContent, setEditContent] = useState(reply.content);

    const handleEditClick = () => {
        setIsEditing(true);
    };

    const handleEditCancel = () => {
        setIsEditing(false);
        setEditContent(reply.content);
    };

    const handleEditSave = async () => {
        if (!editContent.trim() || !accessToken || !userId) return;

        try {

            const updatedReply = await PostService.updateReply(reply.replyToId, reply.id, editContent, accessToken.value);
            if (!updatedReply) {
                console.error("Could not update reply");
                return;
            }

            setReply((prevReply) => {
                if (!prevReply) return prevReply;
                return {
                    ...prevReply,
                    content: updatedReply.content,
                };
            });
            setIsEditing(false);
        } catch (error) {
            console.error("Error updating reply:", error);
        }
    };

    const handleReplySubmit = (newReply: Reply) => {
        setReply((prevReply) => {
            if (!prevReply) return prevReply;
            const updatedNestedReplies = [newReply, ...(prevReply.nestedReplies || [])];
            return {
                ...prevReply,
                nestedReplies: updatedNestedReplies,
            };
        });
    };

    return (
        <div>
            <div className="flex items-center mb-4">
                <Link to={`/profile/${reply.user.id}`}>
                    <img
                        src={reply.user.profileImage.url}
                        alt={reply.user.profileImage.fileName}
                        className="rounded-full w-12 h-12 object-cover"
                    />
                </Link>
                <div className="ml-3">
                    <p className="font-semibold text-lg">{reply.user.name}</p>
                    <p className="text-sm text-gray-500">{relativeDate}</p>
                </div>
                {reply.user.id === userId && !isEditing && (
                    <Button onClick={handleEditClick} className="ml-auto" size="icon" variant="link">
                        Edit
                    </Button>
                )}
            </div>

            {!isEditing ? (
                <p className="mb-4">{reply.content}</p>
            ) : (
                <div className="mt-2">
          <Input
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              className="w-full p-2 border rounded-md"
          />
                    <div className="flex space-x-2 mt-2">
                        <Button
                            onClick={handleEditSave} >
                            Save
                        </Button>
                        <Button
                            onClick={handleEditCancel} variant="secondary">
                            Cancel
                        </Button>
                    </div>
                </div>
            )}

            <PostInteractions
                post={reply}
                isTopLevel={true}
                onReplySubmit={handleReplySubmit}
            />
        </div>
    );
};