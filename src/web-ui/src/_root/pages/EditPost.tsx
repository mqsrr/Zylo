import {useEffect, useState} from "react";
import PostForm from "@/components/forms/PostForm.tsx";
import {useParams} from "react-router-dom";
import {Post} from "@/models/Post.ts";
import {EditIcon} from "lucide-react";
import PostService from "@/services/PostService.ts";
import {useUserContext} from "@/hooks/useTokenContext.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";

const EditPost = () => {
    const {id} = useParams<{ id: string }>();
    const [post, setPost] = useState<Post | null>(null);
    const {user} = useUserContext();
    const {accessToken} = useAuthContext();


    useEffect(() => {
        if (!id || !user || !accessToken) {
            return;
        }

        const initializePost = async (): Promise<void> => {
            const post = await PostService.getPost(id, user.id, accessToken.value);
            if (!post) {
                return;
            }

            setPost(post);
        }

        initializePost().catch(console.error)
    }, [id, user, accessToken]);

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