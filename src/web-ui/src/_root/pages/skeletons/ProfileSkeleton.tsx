import { Skeleton } from "@/components/ui/skeleton.tsx";

const ProfileSkeleton = () => {
    return (
        <div className="container mx-auto px-4 py-6 overflow-auto">
            <div className="relative mb-6">
                <Skeleton className="w-full h-64 object-cover rounded-lg" />
                <div className="absolute -bottom-12 left-6">
                    <Skeleton className="w-24 h-24 object-cover rounded-full border-4 border-white" />
                </div>
            </div>

            <div className="mt-16 mb-8 px-6">
                <Skeleton className="w-48 h-8 mb-2" />
                <Skeleton className="w-32 h-6 mb-4" />

                <div className="flex gap-4 mb-4">
                    <Skeleton className="w-24 h-10 rounded-md" />
                    <Skeleton className="w-24 h-10 rounded-md" />
                </div>

                <Skeleton className="w-1/2 h-5 mb-2" />
                <Skeleton className="w-1/3 h-5 mb-2" />
                <div className="space-y-2 mt-4">
                    <Skeleton className="w-full h-4" />
                    <Skeleton className="w-5/6 h-4" />
                    <Skeleton className="w-2/3 h-4" />
                </div>

                <div className="flex gap-4 mt-6">
                    <Skeleton className="w-20 h-6" />
                    <Skeleton className="w-20 h-6" />
                    <Skeleton className="w-20 h-6" />
                </div>
            </div>

            <div className="px-6">
                <Skeleton className="w-32 h-6 mb-4" />
                <div className="space-y-4">
                    {[...Array(3)].map((_, index) => (
                        <div key={index} className="p-4 border-b border-gray-200">
                            <div className="flex items-start space-x-4">
                                <Skeleton className="w-12 h-12 rounded-full" />
                                <div className="flex-1 space-y-2">
                                    <Skeleton className="w-1/4 h-4" />
                                    <Skeleton className="w-3/4 h-4" />
                                    <Skeleton className="w-full h-4" />
                                    <Skeleton className="w-5/6 h-4" />
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export default ProfileSkeleton;
