export interface AuthenticationResult {
    success: boolean;
    id?: string;
    emailVerified: boolean;
    accessToken?: AccessToken;
    error?: string;
}

export interface AccessToken {
    value: string;
    expirationDate: Date;
}