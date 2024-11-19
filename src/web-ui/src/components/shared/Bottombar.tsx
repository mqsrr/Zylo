import {Link, useLocation} from "react-router-dom";
import {bottombarLinks} from "@/constants/links.ts";
import {NavLink as NavLinkType} from "@/models/NavLink.ts";
import {useUserContext} from "@/hooks/useTokenContext.ts";

const Bottombar = () => {
    const { pathname } = useLocation();
    const { user } = useUserContext();

    return (
        <nav className="md:hidden fixed bottom-0 left-0 right-0 bg-secondary flex justify-around items-center h-16 shadow-lg">
            {bottombarLinks.map((link: NavLinkType) => {
                const isActive =
                    pathname === link.route ||
                    (link.route.startsWith("/profile") && pathname.startsWith("/profile"));

                const IconComponent = link.icon;

                return (
                    <Link
                        to={
                            link.route === "/profile" && user
                                ? `/profile/${user.id}`
                                : link.route
                        }
                        key={link.label}
                        className={`flex flex-col items-center justify-center ${
                            isActive ? "" : "text-gray-400"
                        }`}
                    >
                        <IconComponent size={24} />
                        <span className="text-xs">{link.label}</span>
                    </Link>
                );
            })}
        </nav>
    );
};


export default Bottombar;