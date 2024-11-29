import {Card, CardContent} from "@/components/ui/card.tsx";
import {Form, FormControl, FormField, FormItem, FormLabel, FormMessage} from "@/components/ui/form.tsx";
import {Input} from "@/components/ui/input.tsx";
import {Button} from "@/components/ui/button.tsx";
import {Link, useNavigate} from "react-router-dom";
import {useForm} from "react-hook-form";
import {z} from "zod";
import {SignInFormValidationSchema} from "@/lib/validation";
import {zodResolver} from "@hookform/resolvers/zod";
import {useState} from "react";
import {toast} from "@/hooks/use-toast.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";

const SignInForm = () => {
    const [isLoading, setIsLoading] = useState(false);
    const {signIn} = useAuthContext();
    const navigate = useNavigate();

    const form = useForm<z.infer<typeof SignInFormValidationSchema>>({
        resolver: zodResolver(SignInFormValidationSchema),
        defaultValues: {
            username: "",
            password: "",
        },
    });

    const onSubmit = async (values: z.infer<typeof SignInFormValidationSchema>) => {
            setIsLoading(true);
            const authResult = await signIn(values.username, values.password);

            setIsLoading(false);
            if (!authResult) {
                toast({
                    variant: "destructive",
                    title: "Error",
                    description: `Login failed`,
                });
                return;
            }

            form.reset();
            if (!authResult.emailVerified && authResult.success) {
                navigate("/verify-email");
                return;
            }

            navigate("/");
    };

    return (
        <Card>
            <CardContent>
                <Form {...form}>
                    <div className="sm:w-420 flex-center flex-col">
                        <h2 className="h3-bold md:h2-bold pt-5 sm:pt-12 mt-2"> Log in into your account</h2>
                        <form onSubmit={form.handleSubmit(onSubmit)} className="flex-col gap-5 w-full mt-6 space-y-3">
                            <FormField
                                control={form.control}
                                name="username"
                                render={({field}) => (
                                    <FormItem>
                                        <FormLabel>Username</FormLabel>
                                        <FormControl>
                                            <Input placeholder="Username" type="text" {...field} />
                                        </FormControl>
                                        <FormMessage/>
                                    </FormItem>
                                )}
                            />
                            <FormField
                                control={form.control}
                                name="password"
                                render={({field}) => (
                                    <FormItem>
                                        <FormLabel>Password</FormLabel>
                                        <FormControl>
                                            <Input placeholder="Password1234" type="password" {...field} />
                                        </FormControl>
                                        <FormMessage/>
                                    </FormItem>
                                )}
                            />
                            <Button type="submit" className="w-full">
                                {isLoading ? (
                                    <div className="flex-center gap-2">
                                        Loading...
                                    </div>
                                ) : (
                                    "Login"
                                )}
                            </Button>
                            <p className="text-small-regular text-light-2 text-center mt-2">
                                Still without an account?
                                <Link to="/sign-up" className="text-small-semibold ml-1 underline">Sign up!</Link>
                            </p>
                        </form>
                    </div>
                </Form>
            </CardContent>
        </Card>
    );
};

export default SignInForm;