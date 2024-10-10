import React from "react";
import { useLoginOrNull } from "./LoginProvider";

const WelcomeScreen = () => {
    const login = useLoginOrNull();
    return (
        <div>
            {login.loginData ? (
                <div>
                    <h1>Welcome</h1>
                    <div>{login.loginData.email}</div>
                </div>
            ) : (
                <div>
                    <h1>Log in needed!</h1>
                </div>
            )}
        </div>
    );
};
export default WelcomeScreen;
