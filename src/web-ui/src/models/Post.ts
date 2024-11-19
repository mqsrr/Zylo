import {FileMetadata} from "@/models/FileMetadata.ts";
import {Reply} from "@/models/Reply.ts";

export interface Post {
    id: string;
    text: string;
    likes: number;
    views: number;
    user: UserPost;
    filesMetadata: FileMetadata[] | null;
    replies: Reply[] | null;
    createdAt: string;
    updatedAt: string;
    userInteracted: boolean;
}

export interface UserPost {
    id: string;
    profileImage: FileMetadata;
    name: string;
    username: string;
    bio: string;
    location: string;
}

