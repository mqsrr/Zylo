import {Button} from "@/components/ui/button.tsx";

import {z} from "zod"
import {zodResolver} from "@hookform/resolvers/zod";
import {useForm} from "react-hook-form";
import {
    Form,
    FormControl,
    FormField,
    FormItem, FormLabel,
    FormMessage
} from "@/components/ui/form.tsx";
import {Input} from "@/components/ui/input.tsx";
import {SignUpFormValidationSchema} from "@/lib/validation";
import {Card, CardContent} from "@/components/ui/card.tsx";
import {Link, useNavigate} from "react-router-dom";
import React, {useState} from "react";
import {toast} from "@/hooks/use-toast.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {Loader2} from "lucide-react";

const SignUpForm = () => {
    const [isLoading, setIsLoading] = useState(false);
    const [profile, setProfile] = useState<File | null>(null);
    const [profilePreview, setProfilePreview] = useState<string | null>(null);
    const [background, setBackground] = useState<File | null>(null);
    const [backgroundPreview, setBackgroundPreview] = useState<string | null>(null);
    const {signUp} = useAuthContext();
    const navigate = useNavigate();

    const form = useForm<z.infer<typeof SignUpFormValidationSchema>>({
        resolver: zodResolver(SignUpFormValidationSchema),
        defaultValues: {
            name: "",
            username: "",
            email: "",
            password: "",
            birthdate: "",
        },
    });

    const handleFileChange = (
        event: React.ChangeEvent<HTMLInputElement>,
        setter: (file: File | null) => void,
        previewSetter: (url: string | null) => void
    ) => {
        const file = event.target.files ? event.target.files[0] : null;
        setter(file);

        if (file) {
            const reader = new FileReader();
            reader.onloadend = () => {
                previewSetter(reader.result as string);
            };
            reader.readAsDataURL(file);
        } else {
            previewSetter(null);
        }
    };

    const onSubmit = async (values: z.infer<typeof SignUpFormValidationSchema>) => {
        setIsLoading(true);

        const formData = new FormData();
        formData.append("name", values.name);
        formData.append("username", values.username);
        formData.append("email", values.email);
        formData.append("password", values.password);


        formData.append("birthdate", values.birthdate);

        if (profile) {
            formData.append("ProfileImage", profile);
        }
        if (background) {
            formData.append("BackgroundImage", background);
        }

        const result = await signUp(formData);
        setIsLoading(false);
        if (!result) {
            toast({
                variant: "destructive",
                title: "Error",
                description: `Signup failed`,
            });

            return;
        }

        form.reset();
        navigate(`/verify-email?username=${values.username}&password=${values.password}`);
    };

    return (
        <div className="flex justify-center items-center min-h-screen">
            <Card className="w-full max-w-2xl">
                <CardContent>
                    <Form {...form}>
                        <div className="flex flex-col items-center">
                            <h2 className="text-2xl font-bold mt-4">Create a new account</h2>

                            <div className="relative w-full mt-6">
                                <label htmlFor="background-upload">
                                    <div className="w-full h-48 bg-gray-200 rounded-lg overflow-hidden cursor-pointer">
                                        {backgroundPreview ? (
                                            <img
                                                src={backgroundPreview}
                                                alt="Background Preview"
                                                className="w-full h-full object-cover"
                                            />
                                        ) : (
                                            <div className="w-full h-full flex items-center justify-center text-gray-500">
                                                Click to upload background image
                                            </div>
                                        )}
                                    </div>
                                    <input
                                        id="background-upload"
                                        type="file"
                                        accept="image/*"
                                        className="hidden"
                                        onChange={(e) =>
                                            handleFileChange(e, setBackground, setBackgroundPreview)
                                        }
                                    />
                                </label>
                                {/* Profile Image Preview */}
                                <label
                                    htmlFor="profile-upload"
                                    className="absolute -bottom-10 left-1/2 transform -translate-x-1/2 cursor-pointer"
                                >
                                    {profilePreview ? (
                                        <img
                                            src={profilePreview}
                                            alt="Profile Preview"
                                            className="w-20 h-20 rounded-full object-cover border-4 border-white"
                                        />
                                    ) : (
                                        <div className="w-20 h-20 rounded-full bg-gray-300 flex items-center justify-center border-4 border-white text-gray-500">
                                            Profile
                                        </div>
                                    )}
                                    <input
                                        id="profile-upload"
                                        type="file"
                                        accept="image/*"
                                        className="hidden"
                                        onChange={(e) =>
                                            handleFileChange(e, setProfile, setProfilePreview)
                                        }
                                    />
                                </label>
                            </div>

                            <form
                                onSubmit={form.handleSubmit(onSubmit)}
                                className="flex flex-col gap-4 w-full mt-16"
                            >
                                <div className="flex flex-col md:flex-row gap-4">
                                    <FormField
                                        control={form.control}
                                        name="name"
                                        render={({ field }) => (
                                            <FormItem className="flex-1">
                                                <FormLabel>Name</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="Your Name" type="text" {...field} />
                                                </FormControl>
                                                {/* Reserve space for FormMessage */}
                                                <div className="h-5">
                                                    <FormMessage />
                                                </div>
                                            </FormItem>
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="username"
                                        render={({ field }) => (
                                            <FormItem className="flex-1">
                                                <FormLabel>Username</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="Username" type="text" {...field} />
                                                </FormControl>
                                                <div className="h-5">
                                                    <FormMessage />
                                                </div>
                                            </FormItem>
                                        )}
                                    />
                                </div>

                                <div className="flex flex-col md:flex-row gap-4">
                                    <FormField
                                        control={form.control}
                                        name="email"
                                        render={({ field }) => (
                                            <FormItem className="flex-1">
                                                <FormLabel>Email</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="email@example.com" type="email" {...field} />
                                                </FormControl>
                                                <div className="h-5">
                                                    <FormMessage />
                                                </div>
                                            </FormItem>
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="password"
                                        render={({ field }) => (
                                            <FormItem className="flex-1">
                                                <FormLabel>Password</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="Password" type="password" {...field} />
                                                </FormControl>
                                                <div className="h-5">
                                                    <FormMessage />
                                                </div>
                                            </FormItem>
                                        )}
                                    />
                                </div>

                                <FormField
                                    control={form.control}
                                    name="birthdate"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>Date of Birth</FormLabel>
                                            <FormControl>
                                                <Input type="date" {...field} />
                                            </FormControl>
                                            <div className="h-5">
                                                <FormMessage />
                                            </div>
                                        </FormItem>
                                    )}
                                />

                                <Button type="submit" className="w-full mt-4" disabled={isLoading}>
                                    {isLoading ? (
                                        <div className="flex items-center justify-center gap-2">
                                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                            Signing Up...
                                        </div>
                                    ) : (
                                        "Sign Up"
                                    )}
                                </Button>
                                <p className="text-center mt-2">
                                    Already have an account?
                                    <Link to="/sign-in" className="ml-1 underline">
                                        Log in
                                    </Link>
                                </p>
                            </form>
                        </div>
                    </Form>
                </CardContent>
            </Card>
        </div>
    );};

export default SignUpForm;