
import { Skeleton } from "@/components/ui/skeleton";
import { Card, CardContent, CardHeader } from "@/components/ui/card";

const ReplyDetailSkeleton = () => {
    return (
        <div className="container mx-auto px-4 py-6 overflow-auto">
            <Card className="mb-6">
                <CardHeader>
                    <Skeleton className="w-32 h-6" />
                </CardHeader>
                <CardContent>
                    <div className="flex items-center mb-4">
                        <Skeleton className="rounded-full w-12 h-12" />
                        <div className="ml-3">
                            <Skeleton className="w-32 h-5 mb-2" />
                            <Skeleton className="w-24 h-4" />
                        </div>
                    </div>
                    <Skeleton className="w-full h-4 mb-2" />
                    <Skeleton className="w-5/6 h-4 mb-2" />
                    <Skeleton className="w-3/4 h-4 mb-4" />
                    <div className="flex space-x-4">
                        <Skeleton className="w-16 h-6" />
                        <Skeleton className="w-16 h-6" />
                        <Skeleton className="w-16 h-6" />
                    </div>
                </CardContent>
            </Card>

            <div className="space-y-4">
                {[...Array(2)].map((_, index) => (
                    <Card key={index}>
                        <CardContent>
                            <div className="flex items-center mb-4">
                                <Skeleton className="rounded-full w-10 h-10" />
                                <div className="ml-3">
                                    <Skeleton className="w-28 h-5 mb-1" />
                                    <Skeleton className="w-20 h-4" />
                                </div>
                            </div>
                            <Skeleton className="w-full h-4 mb-2" />
                            <Skeleton className="w-4/5 h-4 mb-2" />
                            <Skeleton className="w-2/3 h-4 mb-4" />
                            <div className="flex space-x-4">
                                <Skeleton className="w-14 h-6" />
                                <Skeleton className="w-14 h-6" />
                                <Skeleton className="w-14 h-6" />
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
        </div>
    );
};

export default ReplyDetailSkeleton;
