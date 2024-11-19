import { Skeleton } from "@/components/ui/skeleton.tsx";

const PostCardSkeleton = () => {
    return (
        <div className="p-4 border-b border-gray-200">
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
    );
};

export default PostCardSkeleton;