﻿import {ImagePlus} from "lucide-react";
import PostForm from "@/components/forms/PostForm.tsx";

const CreatePost = () => {
    return (
        <div className="flex flex-1">
            <div className="common-container">
                <div className="max-w-5xl flex-start gap-3 justify-start w-full">
                    <ImagePlus/>
                    <h2 className="h3-bold md:h2-bold text-left w-full"> Create Post</h2>
                </div>
                <PostForm/>

            </div>
        </div>
    );
};

export default CreatePost;