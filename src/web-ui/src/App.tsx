import {Routes, Route} from "react-router-dom";

import "./globals.css"
import SignUpForm from "./_auth/forms/SignUpForm.tsx";
import SignInForm from "./_auth/forms/SignInForm.tsx";
import {CreatePost, EditPost, Home, PostDetails, Profile, UpdateProfile, ReplyDetails, Social} from "./_root/pages";
import AuthLayout from "./_auth/AuthLayout.tsx";
import RootLayout from "./_root/RootLayout.tsx";
import {Toaster} from "@/components/ui/toaster.tsx";
import OtpVerification from "@/_auth/forms/OtpVerification.tsx";

export default function App() {
    return (
        <main className="flex h-screen bg-primary-foreground text-primary dark">
            <Routes>
                <Route element={<AuthLayout/>}>
                    <Route path="/sign-up" element={<SignUpForm/>}/>
                    <Route path="/sign-in" element={<SignInForm/>}/>
                    <Route path="/verify-email" element={<OtpVerification/>}/>
                </Route>

                <Route element={<RootLayout/>}>
                    <Route index path="/" element={<Home/>}/>
                    <Route path="/create-post" element={<CreatePost/>}/>
                    <Route path="/edit/posts/:id" element={<EditPost/>}/>
                    <Route path="/posts/:id" element={<PostDetails/>}/>
                    <Route path="/posts/:postId/replies/:replyId" element={<ReplyDetails/>}/>
                    <Route path="/profile/:id" element={<Profile/>}/>
                    <Route path="/profile/:id/socials" element={<Social/>}/>
                    <Route path="/update-profile/:id" element={<UpdateProfile/>}/>
                    {/*<Route path="/notifications" element={<Notifications/>}/>*/}
                </Route>
            </Routes>
            <Toaster/>
        </main>
    )
}

