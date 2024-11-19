import {useState} from "react";
import UserService from "@/services/UserService";
import { useNavigate } from "react-router-dom";
import {Button} from "@/components/ui/button.tsx";
import {useUserContext} from "@/hooks/useTokenContext.ts";

const EditProfile = () => {
    const { user } = useUserContext();
    const [formData, setFormData] = useState({
        name: "",
        username: "",
        bio: "",
        location: "",
        birthDate: "",
        profileImage: null,
        backgroundImage: null,
    });
    const [isSubmitting, setIsSubmitting] = useState(false);
    const navigate = useNavigate();


    const handleChange = (e) => {
        const { name, value, files } = e.target;
        if (files) {
            setFormData((prevData) => ({
                ...prevData,
                [name]: files[0],
            }));
        } else {
            setFormData((prevData) => ({
                ...prevData,
                [name]: value,
            }));
        }
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        setIsSubmitting(true);

        try {
            // Prepare data for submission
            const updateData = new FormData();
            Object.keys(formData).forEach((key) => {
                if (formData[key]) {
                    updateData.append(key, formData[key]);
                }
            });

            // Call the update API
            await UserService.updateUser(userId, accessToken.value, updateData);

            // Redirect back to the profile page
            navigate(`/profile/${userId}`);
        } catch (error) {
            console.error("Failed to update profile:", error);
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <div className="container mx-auto px-4 py-6">
            <h1 className="text-2xl font-bold mb-4">Edit Profile</h1>
            <form onSubmit={handleSubmit} className="space-y-4">
                {/* Name Field */}
                <div>
                    <label className="block text-sm font-medium text-gray-700">Name</label>
                    <input
                        type="text"
                        name="name"
                        value={formData.name}
                        onChange={handleChange}
                        className="mt-1 block w-full border border-gray-300 rounded-md"
                    />
                </div>
                {/* Username Field */}
                <div>
                    <label className="block text-sm font-medium text-gray-700">Username</label>
                    <input
                        type="text"
                        name="username"
                        value={formData.username}
                        onChange={handleChange}
                        className="mt-1 block w-full border border-gray-300 rounded-md"
                    />
                </div>
                {/* Bio Field */}
                <div>
                    <label className="block text-sm font-medium text-gray-700">Bio</label>
                    <textarea
                        name="bio"
                        value={formData.bio}
                        onChange={handleChange}
                        className="mt-1 block w-full border border-gray-300 rounded-md"
                    />
                </div>
                {/* Location Field */}
                <div>
                    <label className="block text-sm font-medium text-gray-700">Location</label>
                    <input
                        type="text"
                        name="location"
                        value={formData.location}
                        onChange={handleChange}
                        className="mt-1 block w-full border border-gray-300 rounded-md"
                    />
                </div>
                {/* Birth Date Field */}
                <div>
                    <label className="block text-sm font-medium text-gray-700">Birth Date</label>
                    <input
                        type="date"
                        name="birthDate"
                        value={formData.birthDate}
                        onChange={handleChange}
                        className="mt-1 block w-full border border-gray-300 rounded-md"
                    />
                </div>
                {/* Profile Image Field */}
                <div>
                    <label className="block text-sm font-medium text-gray-700">Profile Image</label>
                    <input
                        type="file"
                        name="profileImage"
                        accept="image/*"
                        onChange={handleChange}
                        className="mt-1 block w-full"
                    />
                </div>
                {/* Background Image Field */}
                <div>
                    <label className="block text-sm font-medium text-gray-700">Background Image</label>
                    <input
                        type="file"
                        name="backgroundImage"
                        accept="image/*"
                        onChange={handleChange}
                        className="mt-1 block w-full"
                    />
                </div>
                {/* Submit Button */}
                <div>
                    <Button type="submit" disabled={isSubmitting}>
                        {isSubmitting ? "Updating..." : "Update Profile"}
                    </Button>
                </div>
            </form>
        </div>
    );
};

export default EditProfile;
