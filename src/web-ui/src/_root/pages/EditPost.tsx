import {useCallback, useEffect, useState} from "react";
import PostForm from "@/components/forms/PostForm.tsx";
import {useParams} from "react-router-dom";
import {Post} from "@/models/Post.ts";
import {usePostContext} from "@/hooks/usePostContext.ts";
import {EditIcon} from "lucide-react";

const EditPost = () => {
    const {id} = useParams<{ id: string }>();
    const [post, setPost] = useState<Post | null>(null);
    const {feed} = usePostContext();

    const findPostById = useCallback((id: string): Post | null => {
            const post = feed.find((post) => post.id === id);
            if (!post) {
                return null;
            }

            return post
        },
        [feed]
    );

    useEffect(() => {
        if (!id) {
            return;
        }

        const post = findPostById(id);
        if (!post) {
            console.error('Could not find reply!');
            return;
        }
        setPost(post);
    }, [id, findPostById]);

    if (!post) {
        return <div>Loading...</div>;
    }

    return <div className="flex flex-1">
        <div className="common-container">
            <div className="max-w-5xl flex-start gap-3 justify-start w-full">
                <EditIcon/>
                <h2 className="h3-bold md:h2-bold text-left w-full"> Edit Post</h2>
            </div>
            <PostForm post={post} isEditing={true}/>

        </div>
    </div>
};

export default EditPost;