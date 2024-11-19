import {Link} from "react-router-dom";
import {HomeIcon, LogOutIcon, UserIcon} from "lucide-react";
import {Button} from "@/components/ui/button.tsx";
import {useUserContext} from "@/hooks/useTokenContext.ts";
import {useAuthContext} from "@/hooks/useAuthContext.ts";

const Topbar = () => {
    const {user} = useUserContext();
    const {logout} = useAuthContext();

    return (
        <header
            className="md:hidden fixed top-0 left-0 right-0 h-16 bg-secondary flex items-center justify-between px-4 shadow-md z-50 ">
            <div className="flex items-center gap-4">
                <Link to="/" className="flex items-center gap-2">
                    <HomeIcon size={24}/>
                    <span className="text-xl font-bold">Zylo Platform</span>
                </Link>
            </div>

            <div className="flex items-center gap-4">
                {user && (
                    <Link to={`/profile/${user.id}`} className="flex items-center">
                        <UserIcon size={24} className="text-white"/>
                    </Link>
                )}
                <Button variant="ghost" onClick={logout} aria-label="Logout">
                    <LogOutIcon size={24}/>
                </Button>
            </div>
        </header>
    );
};

export default Topbar;