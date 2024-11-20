import React, {createContext, useEffect, ReactNode} from 'react';
import * as signalR from '@microsoft/signalr';
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {useUserContext} from "@/hooks/useTokenContext.ts";

export const NotificationsContext = createContext(undefined);

export const NotificationsProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const {accessToken} = useAuthContext();
    const {user} = useUserContext();

    useEffect(() => {
        if (!accessToken) {
            return;
        }

        const connection = new signalR.HubConnectionBuilder()
            .withUrl('http://localhost:8090/notifications', {
                accessTokenFactory(): string | Promise<string> {
                    return accessToken.value
                }
            })
            .withAutomaticReconnect()
            .build();

        connection
            .start()
            .then(() => {

                user?.relationships.friends?.forEach(friend => {
                    connection.invoke("JoinFriendGroup", friend.id)
                        .catch(console.error)
                })

                user?.relationships.followedPeople?.forEach(followed => {
                    connection.invoke("JoinFollowerGroup", followed.id)
                        .catch(console.error)
                })

                connection.on("UserBlocked", () => console.log("One of the user has blocked you!"))
                connection.on("UserUnblocked", () => console.log("Someone unblocked you!"))
                connection.on("UserFollowed", () => console.log("Someone followed you!"))
                connection.on("UserUnfollowed", () => console.log("Someone unfollowed you!"))

                connection.on("FriendRequestAccepted", () => console.log("Friend request has been accepted!"))
                connection.on("FriendRequestDeclined", () => console.log("Friend request has been declined!"))
                connection.on("FriendRemoved", () => console.log("Someone removed you from friends list ("))
                connection.on("FriendRequestSent", () => console.log("You received the friend request"))

                connection.on("PostLiked", () => console.log("On of your post just got liked"))

            })
            .catch((error) => console.error('Connection error: ', error));

        return () => {
            user?.relationships.friends?.forEach(friend => {
                connection.invoke("LeaveFriendGroup", friend.id)
                    .catch(console.error)
            })

            user?.relationships.followedPeople?.forEach(followed => {
                connection.invoke("LeaveFollowerGroup", followed.id)
                    .catch(console.error)
            })

            connection.stop()
                .catch(console.error);
        };
    }, [accessToken, user]);


    return (
        <NotificationsContext.Provider value={undefined}>
            {children}
        </NotificationsContext.Provider>
    );
};
