import Topbar from "@/components/shared/Topbar.tsx";
import LeftSidebar from "@/components/shared/LeftSidebar.tsx";
import Bottombar from "@/components/shared/Bottombar.tsx";
import {Outlet} from "react-router-dom";

const RootLayout = () => {
    return (
        <div className="w-full md:flex">
            <Topbar />
            <LeftSidebar/>

            <section className="flex flex-1 h-full sm:my-16 md:my-0">
                <Outlet />
            </section>

            <Bottombar/>
        </div>
    );
};

export default RootLayout;