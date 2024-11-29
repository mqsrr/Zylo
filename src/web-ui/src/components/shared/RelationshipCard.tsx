import { Link } from "react-router-dom";
import UserService from "@/services/UserService.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {useCallback, useEffect} from "react";
import {Card, CardContent} from "@/components/ui/card.tsx";
import {UserSummary} from "@/models/User.ts";

type RelationshipCardProps = {
    title: string;
    users: UserSummary[] | null;
};

const RelationshipCard = ({ title, users }: RelationshipCardProps) => {
    const {accessToken} = useAuthContext()
    
    const mapUsers = useCallback(async (users: UserSummary[] | null) => {
        if (!users) {
            return;
        }

        for (const user of users) {
            const userData = await UserService.getUser(user.id, accessToken!.value, null)
            user.profileImage.url = userData!.profileImage.url
        }
    }, [accessToken]);
    
    useEffect(() => {
        mapUsers(users).catch(console.error);
    }, [mapUsers, users])
    

    return (
        <div className="w-full">
            <Card className="bg-card-foreground border-card-foreground text-card mb-4">
                <CardContent>
                    <h3 className="text-xl font-semibold mb-4">{title}</h3>
                    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
                        {users?.map((user) => (
                            <div key={user.id} className="flex items-center space-x-4">
                                <Link to={`/profile/${user.id}`} className="flex items-center">
                                    <img
                                        src={user.profileImage.url!}
                                        alt={user.profileImage.url!}
                                        className="w-12 h-12 rounded-full object-cover"
                                    />
                                    <span className="ml-2 text-gray-200 hover:underline">
                    {user.username}
                  </span>
                                </Link>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>
        </div>
    );
};

export default RelationshipCard;
