import {z} from "zod";


export const SignUpFormValidationSchema = z.object({
    name: z.string().min(2, {message: "Name is too short"}).max(50, {message: "Name is too long"}),
    username: z.string().min(2, {message: "Username is too short"}).max(50, {message: "Username is too long"}),
    email: z.string().email({message: "Please enter a valid email"}),
    password: z.string().min(8, {message: "Password must be at least 8 characters."}).max(50, {message: "Password is too long"}),
    birthdate: z.string().min(0,"Birthdate is required"),
})

export const SignInFormValidationSchema = z.object({
    username: z.string().min(2, {message: "Username is too short"}).max(50, {message: "Username is too long"}),
    password: z.string().min(8, {message: "Password must be at least 8 characters."}).max(50, {message: "Password is too long"}),
})

export const CreatePostFormValidationSchema = z.object({
    content: z.string().min(5, {message: "Content is too short"}),
    files: z.custom<File[]>(),
})