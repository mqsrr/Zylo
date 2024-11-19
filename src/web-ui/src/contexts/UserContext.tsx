import React, {createContext, ReactNode, useEffect, useState} from "react";
import {User} from "@/models/User.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import UserService from "@/services/UserService.ts";

interface UserContextType {
    isLoading: boolean;
    user: User | null;
}

export const UserContext = createContext<UserContextType | undefined>(undefined);
export const UserProvider: React.FC<{ children: ReactNode }> = ({children}) => {
    const {accessToken, userId} = useAuthContext();
    const [isLoading, setIsLoading] = useState(false);
    const [user, setUser] = useState<User | null>(null);

    useEffect(() => {
        if (!accessToken || !userId){
            return;
        }
        console.log("fetching data about user")
        const fetchUser = async () => {
            const user = await UserService.getUser(userId, accessToken.value, null);
            if (!user) {
                throw new Error("User could not be found!");
            }

            setUser(user)
        };
        setIsLoading(true);
        fetchUser().catch((error) => console.error(error));
        setIsLoading(false);

    }, [accessToken, userId])


    return (
        <UserContext.Provider value={{
            isLoading,
            user,
        }}>
            {children}
        </UserContext.Provider>
    );
};

