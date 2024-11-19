import {FileMetadata} from "@/models/FileMetadata.ts";

export interface Reply {
    id: string;
    user: UserPost;
    replyToId: string;
    content: string;
    nestedReplies: Reply[];
    likes: number;
    views: number;
    createdAt: Date;
    userInteracted: boolean;
}

