import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage
} from "@/components/ui/form.tsx";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Button } from "@/components/ui/button.tsx";
import { CreatePostFormValidationSchema } from "@/lib/validation";
import { z } from "zod";
import { Textarea } from "@/components/ui/textarea.tsx";
import FileUploader from "@/components/shared/FileUploader.tsx";
import { Post } from "@/models/Post.ts";
import { useNavigate } from "react-router-dom";
import { usePostContext } from "@/hooks/usePostContext";
import PostService from "@/services/PostService.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import React from "react";
import {useUserContext} from "@/hooks/useTokenContext.ts";

interface PostFormProps {
    post?: Post;
    isEditing?: boolean;
}

const PostForm: React.FC<PostFormProps> = ({ post, isEditing = false }) => {
    const { updatePostInFeed } = usePostContext();
    const {user} = useUserContext();
    const {accessToken, userId} = useAuthContext();
    const navigate = useNavigate();

    const form = useForm<z.infer<typeof CreatePostFormValidationSchema>>({
        resolver: zodResolver(CreatePostFormValidationSchema),
        defaultValues: {
            content: post ? post?.text : "",
            files: [],
        },
    });

    const handleCancelClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        navigate("/")
    }

    async function onSubmit(values: z.infer<typeof CreatePostFormValidationSchema>) {
        try {
            const formData = new FormData();
            formData.append("text", values.content);

            values.files.forEach((file) => formData.append("media", file));

            if (isEditing && post) {
                const updatedPost = await PostService.updatePost(userId!, post.id, formData, accessToken!.value);
                if (!updatedPost) {
                    return;
                }

                updatePostInFeed(updatedPost);
                user!.posts.data.map((post) => {
                    if (post.id === updatedPost.id){
                        return updatedPost;
                    }

                    return post;
                })

            } else {
                const createdPost = await PostService.createPost(userId!, formData, accessToken!.value);
                if (!createdPost){
                    console.error("Failed to create post");
                    return;
                }

                user!.posts.data.push(createdPost);
            }

            navigate("/");
        } catch (e) {
            console.error("Failed to submit post", e);
        }
    }
    return (
        <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-col gap-9 w-full max-w-5xl">
                <FormField
                    control={form.control}
                    name="content"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="text-primary-foreground">Content</FormLabel>
                            <FormControl>
                                <Textarea
                                    className="shad-textarea custom-scrollbar"
                                    placeholder="How is your day?"
                                    {...field}
                                />
                            </FormControl>
                            <FormMessage className="text-red" />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="files"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="text-primary-foreground">Add Photos</FormLabel>
                            <FormControl>
                                <FileUploader
                                    fieldChange={field.onChange}
                                />
                            </FormControl>
                            <FormMessage className="text-red" />
                        </FormItem>
                    )}
                />

                <div className="flex gap-4 items-center justify-end">
                    <Button type="button" className="shad-button_dark_4" onClick={handleCancelClick}>Cancel</Button>
                    <Button type="submit" className="shad-button_primary whitespace-nowrap">{isEditing ? "Update" : "Submit"}</Button>
                </div>
            </form>
        </Form>
    );
};

export default PostForm;
