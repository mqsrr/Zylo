import {InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot} from "@/components/ui/input-otp.tsx";
import {useCallback, useEffect, useState} from "react";
import {REGEXP_ONLY_DIGITS_AND_CHARS} from "input-otp";
import {useAuthContext} from "@/hooks/useAuthContext.ts";
import AuthService from "@/services/AuthService.ts";
import {useNavigate, useParams} from "react-router-dom";

const OtpVerification = () => {
    const [value, setValue] = useState("")
    const [isLoading, setIsLoading] = useState(false);
    const {userId} = useAuthContext()
    const {username, password} = useParams<{username:string, password: string}>();
    const navigate = useNavigate();


    const verifyCode = useCallback(async (otpCode: string) => {
        if (!userId){
            return;
        }

        setIsLoading(true);
        const codeMatched = await AuthService.verifyEmail(userId, otpCode);
        if (!codeMatched || !username || !password){
            return;
        }

        const result = await AuthService.signIn(username, password);
        if (!result){
            return;
        }

        setIsLoading(false);
        navigate("/")

    }, [userId, username, password, navigate])

    useEffect(() => {
        if (value.length !== 6) {
            return
        }

        verifyCode(value).catch(console.error);
    }, [value, verifyCode]);

    return (
        <div className="flex items-center justify-center h-screen">
            <div className="flex flex-col items-center text-center w-full max-w-md">
                <h1 className="text-2xl font-bold mb-4">Verify Your Account</h1>

                <div className="flex justify-center mb-6">
                    <InputOTP
                        maxLength={6}
                        value={value}
                        pattern={REGEXP_ONLY_DIGITS_AND_CHARS}
                        onChange={(value) => setValue(value.toUpperCase())}>
                        <InputOTPGroup>
                            <InputOTPSlot index={0}/>
                            <InputOTPSlot index={1}/>
                            <InputOTPSlot index={2}/>
                        </InputOTPGroup>
                        <InputOTPSeparator/>
                        <InputOTPGroup>
                            <InputOTPSlot index={3}/>
                            <InputOTPSlot index={4}/>
                            <InputOTPSlot index={5}/>
                        </InputOTPGroup>
                    </InputOTP>
                </div>
                <p className="mb-4">
                    We’ve sent a 6-characters code to your email. Please enter it below to verify your account.
                </p>

                {isLoading && <p className="text-gray-500">Processing...</p>}
            </div>
        </div>
    );
};

export default OtpVerification;