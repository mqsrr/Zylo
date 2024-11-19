import {RefreshTokenUri, RevokeTokenUri, SignInUri, SignUpUri, VerifyEmailByOtpCode} from "@/constants/requestsUri.ts";
import {AuthenticationResult} from "@/models/Authentication.ts";

class AuthService {

    signUp = async (request: FormData): Promise<AuthenticationResult | null> => {
        const response = await fetch(SignUpUri, {
            method: 'POST',
            body: request,
            credentials: 'include'
        });

        return response.ok
            ? await response.json()
            : null;
    }

    signIn = async (username: string, password: string): Promise<AuthenticationResult | null> => {
        const response = await fetch(SignInUri, {
            method: 'POST',
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                "username": username,
                "password": password,
            }),
            credentials: 'include'
        });

        return response.ok
            ? await response.json()
            : null;
    };

    verifyEmail = async (id: string, otpCode: string): Promise<boolean> => {
        const response = await fetch(VerifyEmailByOtpCode(id), {
            method: 'POST',
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                "otp": otpCode
            }),
        });

        return response.ok;
    }

    refreshAccessToken = async (): Promise<AuthenticationResult | null> => {
        const response = await fetch(RefreshTokenUri, {
            method: 'POST',
            credentials: 'include'
        });

        return response.ok
            ? await response.json()
            : null;
    }


    revokeRefreshToken = async (): Promise<boolean> => {
        const response = await fetch(RevokeTokenUri, {
            method: 'POST',
            credentials: 'include'
        });

        return response.ok;
    }
}

export default new AuthService();