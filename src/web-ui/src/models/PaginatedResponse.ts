export interface PaginatedResponse<Type> {
    data: Type[];
    hasNextPage: boolean;
    perPage: number;
    next: string;
}