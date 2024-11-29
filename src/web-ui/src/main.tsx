import {createRoot} from 'react-dom/client'
import {BrowserRouter} from "react-router-dom";
import App from "./App.tsx";
import {AuthProvider} from "@/contexts/AuthContext.tsx";
import {UserProvider} from "@/contexts/UserContext.tsx";
import {NotificationsProvider} from "@/contexts/NotificationContext.tsx";
import {PostProvider} from "@/contexts/PostContext.tsx";

createRoot(document.getElementById('root')!).render(
    <BrowserRouter>
        <AuthProvider>
            <PostProvider>
                <UserProvider>
                    <NotificationsProvider>
                        <App/>
                    </NotificationsProvider>
                </UserProvider>
            </PostProvider>

        </AuthProvider>
    </BrowserRouter>
)
