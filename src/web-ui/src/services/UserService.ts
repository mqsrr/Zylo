import {User} from "@/models/User.ts";
import {DeleteUserUri, GetUserUri, UpdateUserUri} from "@/constants/requestsUri.ts";

class UserService {

    getUser = async (id: string, token: string, currentUserId: string | null): Promise<User | null> => {
        const response = await fetch(GetUserUri(id, currentUserId), {
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok
            ? await response.json()
            : null;
    }

    updateUser = async (id: string, formData: FormData, token: string): Promise<User | null> => {
        const response = await fetch(UpdateUserUri(id), {
            method: 'PUT',
            headers: {
                Authorization: `Bearer ${token}`,
            },
            body: formData,
        });

        return response.ok
            ? await response.json()
            : null;
    }

    deleteUser = async (id: string, token: string): Promise<boolean> => {
        const response = await fetch(DeleteUserUri(id), {
            method: 'DELETE',
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        return response.ok;
    }
}

export default new UserService();