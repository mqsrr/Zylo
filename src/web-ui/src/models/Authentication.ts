export interface AuthenticationResult {
    id: string;
    emailVerified: boolean,
    accessToken: AccessToken;
}

export interface AccessToken {
    value: string;
    expirationDate: Date;
}