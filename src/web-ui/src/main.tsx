import {createRoot} from 'react-dom/client'
import {BrowserRouter} from "react-router-dom";
import App from "./App.tsx";
import {AuthProvider} from "@/contexts/AuthContext.tsx";
import {UserProvider} from "@/contexts/UserContext.tsx";
import {PostsProvider} from "@/contexts/PostContext.tsx";
import {NotificationsProvider} from "@/contexts/NotificationContext.tsx";

createRoot(document.getElementById('root')!).render(
    <BrowserRouter>
        <AuthProvider>
                <UserProvider>
                    <NotificationsProvider>
                        <PostsProvider>
                            <App/>
                        </PostsProvider>
                    </NotificationsProvider>
                </UserProvider>
        </AuthProvider>
    </BrowserRouter>
)
