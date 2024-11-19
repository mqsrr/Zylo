import {Link, NavLink, useLocation} from "react-router-dom";
import {IceCream2Icon, LogOutIcon} from "lucide-react";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import {sidebarLinks} from "@/constants/links.ts";
import {useUserContext} from "@/hooks/useTokenContext.ts";


const LeftSidebar = () => {
    const { logout } = useAuthContext();
    const { user, isLoading } = useUserContext();
    const { pathname } = useLocation();

    if (isLoading || !user) {
        return (
            <nav className="flex flex-col p-6">
                <div className="flex flex-col gap-11">
                    <div>Loading...</div>
                </div>
            </nav>
        );
    }

    return (
        <nav className="hidden md:flex flex-col w-64 h-screen sticky top-0 p-6 bg-dark-3">
            {/* Logo */}
            <div className="flex items-center gap-3 mb-8">
                <IceCream2Icon size={28} />
                <Link to="/" className="text-2xl font-bold">
                    Zylo Platform
                </Link>
            </div>

            {/* User Profile */}
            <Link
                to={`/profile/${user.id}`}
                className="flex items-center gap-3 mb-8 p-2 rounded-lg hover:bg-secondary transition-colors"
            >
                <img
                    src={user.profileImage.url}
                    alt={user.profileImage.fileName}
                    className="h-14 w-14 rounded-full object-cover"
                />
                <div className="flex flex-col">
                    <p className="font-semibold">{user.name}</p>
                    <p className="text-sm text-gray-400">@{user.username}</p>
                </div>
            </Link>

            <ul className="flex flex-col gap-2">
                {sidebarLinks.map((link) => {
                    const isActive = pathname === link.route || pathname.startsWith(link.route, 1);
                    const IconComponent = link.icon;

                    return (
                        <li key={link.label}>
                            <NavLink
                                to={
                                    link.route === "/profile/"
                                        ? `/profile/${user.id}/socials`
                                        : link.route
                                }
                                className={`flex items-center gap-4 p-3 rounded-lg transition-colors ${
                                    isActive
                                        ? "bg-secondary"
                                        : "text-gray-300 bg-dark-3 hover:bg-secondary hover:text-white"
                                }`}
                            >
                                <IconComponent size={24} />
                                <span className="font-medium">{link.label}</span>
                            </NavLink>
                        </li>
                    );
                })}
            </ul>

            <button
                onClick={logout}
                className="flex items-center gap-2 mt-auto text-gray-300 hover:text-white hover:bg-secondary p-3 rounded-lg transition-colors"
            >
                <LogOutIcon size={20} />
                <span className="font-medium">Logout</span>
            </button>
        </nav>
    );
};
export default LeftSidebar;