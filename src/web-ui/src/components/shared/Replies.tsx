import {Reply} from "@/models/Reply.ts";
import ReplyItem from "@/components/shared/ReplyItem.tsx";

type RepliesProps = {
    replies: Reply[];
    postId: string;
    maxExpandLevel: number;
};

const Replies = ({ replies, postId, maxExpandLevel }: RepliesProps) => {
    return (
        <div className="mt-6">
            {replies.map((reply) => (
                <ReplyItem
                    key={reply.id}
                    postId={postId}
                    reply={reply}
                    level={0}
                    maxExpandLevel={maxExpandLevel}
                />
            ))}
        </div>
    );
};
export default Replies;

