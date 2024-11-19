import { Post } from "@/models/Post.ts";
import {FileMetadata} from "@/models/FileMetadata.ts";
import {UserRelationship} from "@/models/UserRelationship.ts";
import {PaginatedResponse} from "@/models/PaginatedResponse.ts";

export interface User {
    id: string;
    username: string;
    profileImage: FileMetadata;
    backgroundImage: FileMetadata;
    name: string;
    bio: string;
    location: string;
    birthDate: Date;
    relationships: UserRelationship;
    posts: PaginatedResponse<Post>;
    createdAt: Date;
}


export interface UserSummary {
    id: string;
    profileImage: FileMetadata;
    name: string;
    username: string;
    bio: string | null;
    location: string | null;
}