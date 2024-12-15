import React, {createContext, ReactNode, useCallback, useEffect, useState} from "react";
import {AccessToken, AuthenticationResult} from "@/models/Authentication.ts";
import {useNavigate} from "react-router-dom";
import AuthService from "@/services/AuthService.ts";

interface AuthContextType {
    accessToken?: AccessToken | null;
    userId?: string;
    refreshAccessToken: () => Promise<AccessToken | null>;
    signUp (request: FormData): Promise<AuthenticationResult | null>;
    signIn (username: string, password: string): Promise<AuthenticationResult | null>;
    logout: () => void;
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined);
export const AuthProvider: React.FC<{ children: ReactNode }> = ({children}) => {
    const [isLoading, setIsLoading] = useState(false);
    const [userId, setUserId] = useState("");
    const [accessToken, setAccessToken] = useState<AccessToken | null>(null);
    const navigate = useNavigate();

    const refreshAccessToken = useCallback(async () => {
        console.log("calling refresh access token");
        setIsLoading(true)

        const response = await AuthService.refreshAccessToken();
        if (!response){
            return null;
        }

        console.log("token refreshed");
        const accessToken = response.accessToken!;

        setAccessToken(accessToken);
        setUserId(response.id!);
        setIsLoading(false)

        return accessToken;
    }, []);

    const signIn = async (username: string, password: string) => {
        setIsLoading(true)
        const response = await AuthService.signIn(username, password);
        if (!response) {
            return null;
        }
        setUserId(response.id!);
        setIsLoading(false);

        if (response.success) {
            setAccessToken(response.accessToken!);
        }

        return response;
    };

    const signUp = async (request: FormData) => {
        setIsLoading(true)
        const response = await AuthService.signUp(request);
        if (!response) {
            return null;
        }

        setUserId(response.id!);
        setIsLoading(false);

        return response;
    };

    const logout = useCallback(async () => {
        if (!accessToken) {
            return;
        }

        await AuthService.revokeRefreshToken();

        setAccessToken(null);
        setUserId("");
    },[accessToken])

    useEffect(() => {
        if (accessToken && accessToken.expirationDate < new Date(Date.now())) {
            console.log("setting up refresh of access token")
            const REFRESH_BUFFER_TIME = 60 * 1000;

            const timeUntilExpiration = new Date(accessToken.expirationDate).getTime() - Date.now();
            const tokenRefreshTimeout = setTimeout(() => {
                console.log("refreshing access token!")
                refreshAccessToken().catch(e => console.error(e));
            }, timeUntilExpiration - REFRESH_BUFFER_TIME);

            return () => clearTimeout(tokenRefreshTimeout);
        }
    }, [accessToken, refreshAccessToken]);

    useEffect(() => {
        console.log("Refreshing token from the start!");
        refreshAccessToken().catch(e => console.error(e));
    }, [refreshAccessToken]);

    useEffect(() => {
        const path = window.location.pathname;
        const isAuthPath = path === "/sign-in" || path === "/sign-up" || path === "/verify-email"

        if (accessToken && isAuthPath){
            navigate("/");
        }
        else if (!accessToken && !isLoading && !isAuthPath) {
            navigate("/sign-in")
        }
    }, [accessToken, isLoading, navigate]);

    return (
        <AuthContext.Provider value={{
            accessToken,
            userId,
            logout,
            refreshAccessToken,
            signIn,
            signUp,
        }}>
            {children}
        </AuthContext.Provider>
    );
};

