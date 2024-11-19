import {createRoot} from 'react-dom/client'
import {BrowserRouter} from "react-router-dom";
import App from "./App.tsx";
import {AuthProvider} from "@/contexts/AuthContext.tsx";
import {UserProvider} from "@/contexts/UserContext.tsx";
import {PostsProvider} from "@/contexts/PostContext.tsx";

createRoot(document.getElementById('root')!).render(
    <BrowserRouter>
        <AuthProvider>
            <UserProvider>
                <PostsProvider>
                    <App/>
                </PostsProvider>
            </UserProvider>
        </AuthProvider>
    </BrowserRouter>
)
