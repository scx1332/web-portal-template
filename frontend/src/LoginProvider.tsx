import React, { createContext, useContext, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { backendFetch } from "./common/BackendCall";

interface LoginData {
    uid: string;
    email: string;
    createdDate: string;
    lastPassChange: string;
}

export enum LoginStatus {
    LOGGED_IN,
    LOGGED_OUT,
    LOGIN_IN_PROGRESS,
}
interface LoginInformation {
    loginData: LoginData | null;
    loginStatus: LoginStatus;
    loginError: string;
}
export default LoginData;

function defaultLoginInformation(): LoginInformation {
    return {
        loginData: null,
        loginStatus: LoginStatus.LOGIN_IN_PROGRESS,
        loginError: "",
    };
}
interface LogEvent {
    loggedIn: LoginData | null;
    loggedOut: boolean | null;
}

export const LoginContext = createContext<LoginInformation>(defaultLoginInformation());
export const LoginEvent = createContext((e: LogEvent) => {
    console.error("LoginEvent not implemented");
});

export const useLoginEvent = () => useContext(LoginEvent);

export const useLoginOrNull = () => useContext<LoginInformation>(LoginContext);
export const useLogin = () => {
    const value = useLoginOrNull();
    if (!value.loginData) {
        throw new Error("Login not available");
    }
    return value;
};

interface LoginProviderProps {
    children: React.ReactNode;
}

export const LoginProvider = (props: LoginProviderProps) => {
    const [login, setLogin] = useState<LoginInformation>(defaultLoginInformation());
    const navigate = useNavigate();

    const subscribeLoginEvent = (e: LogEvent) => {
        if (e.loggedIn) {
            setLogin({
                loginData: e.loggedIn,
                loginStatus: LoginStatus.LOGGED_IN,
                loginError: "",
            });
            navigate("/");
        } else if (e.loggedOut) {
            setLogin({
                loginData: null,
                loginStatus: LoginStatus.LOGGED_OUT,
                loginError: "",
            });
            navigate("/");
            return;
        } else {
            console.error("Invalid logging event");
            throw new Error("Invalid logging event");
        }
    };
    const greetCheck = async () => {
        const response = await backendFetch("/api/greet", {
            method: "GET",
        });
        const data = await response.json();
        console.log("Web portal backend version:", data.version);
    };

    const [loginCheckInProgress, setLoginCheckInProgress] = useState(false);
    useEffect(() => {
        (async () => {
            setLoginCheckInProgress(true);
            try {
                const response = await backendFetch("/api/is_login", {
                    method: "GET",
                });
                if (response.status === 401) {
                    setLogin({
                        loginData: null,
                        loginStatus: LoginStatus.LOGGED_OUT,
                        loginError: "",
                    });
                    setLoginCheckInProgress(false);
                    return;
                }
                const data = await response.json();
                setLogin({
                    loginData: data,
                    loginStatus: LoginStatus.LOGGED_IN,
                    loginError: "",
                });
                setLoginCheckInProgress(false);
            } catch (e) {
                console.error(e);
                setLoginCheckInProgress(false);
            }
        })();
    }, [setLogin]);

    useEffect(() => {
        greetCheck().then();
    }, []);

    return (
        <LoginEvent.Provider value={subscribeLoginEvent}>
            <LoginContext.Provider value={login}>{props.children}</LoginContext.Provider>
        </LoginEvent.Provider>
    );
};
