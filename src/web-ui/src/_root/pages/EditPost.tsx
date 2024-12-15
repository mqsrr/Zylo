import {useEffect, useState} from "react";
import PostForm from "@/components/forms/PostForm.tsx";
import {useParams} from "react-router-dom";
import {Post} from "@/models/Post.ts";
import {EditIcon} from "lucide-react";
import {usePostContext} from "@/hooks/usePostContext.ts";

const EditPost = () => {
    const {id} = useParams<{ id: string }>();
    const [post, setPost] = useState<Post | null>(null);
    const {getPostById, fetchPostById} = usePostContext()


    useEffect(() => {
        if (!id) {
            return;
        }

        const initializePost = async (): Promise<void> => {
            let post = getPostById(id);
            if (!post) {
               post =  await fetchPostById(id)
            }

            setPost(post);
        }

        initializePost().catch(console.error)
    }, [id, getPostById, fetchPostById]);

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