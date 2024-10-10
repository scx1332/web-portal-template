import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Button, Input, InputLabel, styled, TextField, tooltipClasses, TooltipProps } from "@mui/material";
import Tooltip from "@mui/material/Tooltip";
import { useLoginEvent } from "./LoginProvider";
import { backendFetch } from "./common/BackendCall";

function useWindowSize() {
    const [size, setSize] = useState([0, 0]);
    useLayoutEffect(() => {
        function updateSize() {
            setSize([window.innerWidth, window.innerHeight]);
        }

        window.addEventListener("resize", updateSize);
        updateSize();
        return () => window.removeEventListener("resize", updateSize);
    }, []);
    return size;
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface LoginScreenProps {}

const HtmlTooltip = styled(({ className, ...props }: TooltipProps) => (
    <Tooltip {...props} classes={{ popper: className }} />
))(({ theme }) => ({
    [`& .${tooltipClasses.tooltip}`]: {
        backgroundColor: "#f5f5f9",
        color: "rgba(0, 0, 0, 0.87)",
        maxWidth: 220,
        fontSize: theme.typography.pxToRem(12),
        border: "1px solid #dadde9",
    },
}));

const LoginScreen = (props: LoginScreenProps) => {
    const updateLogin = useLoginEvent();

    const [loginErrorMessage, setLoginErrorMessage] = useState<string>("");
    const [frameCount, setFrameCount] = useState<DOMHighResTimeStamp>(0); // State to track the current frame count
    const requestRef = useRef<number>(); // Holds the requestAnimationFrame id
    const previousTimeRef = useRef<DOMHighResTimeStamp>(); // Holds the previous timestamp

    const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);
    // Animation function that gets called every frame
    const animate = (time: DOMHighResTimeStamp) => {
        if (previousTimeRef.current != undefined) {
            // Calculate the elapsed time between frames
            const deltaTime = time - previousTimeRef.current;

            //Or if you want to update after a specific time (e.g. 60 frames per second)
            //if (deltaTime > 1000 / 60) { // for 60 FPS
            setFrameCount((prevCount) => prevCount + 1);
            //}
        }

        previousTimeRef.current = time; // Update the previous timestamp for the next frame
        requestRef.current = requestAnimationFrame(animate); // Request the next frame
    };
    useEffect(() => {
        isLogin().then();
        //sessionCheck().then();
        // Start the animation loop
        requestRef.current = requestAnimationFrame(animate);

        return () => cancelAnimationFrame(requestRef.current ?? 0); // Cleanup the animation loop on unmount
    }, []); // Empty array to run only once on mount

    const [loginCheckInProgress, setLoginCheckInProgress] = useState<boolean>(false);

    const isLogin = async () => {
        setLoginCheckInProgress(true);
        const response = await backendFetch("/api/is_login", {
            method: "GET",
        });
        const data = await response.text();
        console.log("Is logged in", data);
        if (data === "Logged in") {
            setIsLoggedIn(true);
        }
        setLoginCheckInProgress(false);
    };

    const logout = async () => {
        setLoginCheckInProgress(true);
        const response = await backendFetch("/api/logout", {
            method: "Post",
        });
        const data = await response.text();
        console.log("logout data:", data);
        if (data === "Logged out") {
            setIsLoggedIn(false);
        }
        updateLogin({
            loggedIn: null,
            loggedOut: true,
        });

        setLoginCheckInProgress(false);
    };

    const [loginInProgress, setLoginInProgress] = useState<boolean>(false);
    const loginAction = async () => {
        setLoginInProgress(true);
        setLoginErrorMessage("");
        const response = await backendFetch("/api/login", {
            method: "POST",
            body: JSON.stringify({ email: currentLogin, password: password }),
        });
        if (response.status === 401) {
            setLoginErrorMessage("Wrong user or password");
            setLoginInProgress(false);
            return;
        }
        const data = await response.json();

        updateLogin({
            loggedIn: data,
            loggedOut: false,
        });
        setLoginInProgress(false);
    };

    const currentWidth = useRef<number>(window.innerWidth);
    const currentHeight = useRef<number>(window.innerHeight);

    const targetX = window.innerWidth;
    const targetY = window.innerHeight;

    const newWidth = currentWidth.current + (targetX - currentWidth.current) * 0.1;
    const newHeight = currentHeight.current + (targetY - currentHeight.current) * 0.1;

    /*
    if (Math.abs(newWidth - targetX) / (targetX + newWidth) > 0.2 || Math.abs(newHeight - targetY) / (targetY + newHeight) > 0.2) {
        newWidth = targetX;
        newHeight = targetY;
    }*/

    const [currentLogin, setCurrentLogin] = useState<string>("");
    const [password, setPassword] = useState<string>("");

    currentWidth.current = newWidth;
    currentHeight.current = newHeight;

    const divX = newWidth / 10;
    const divY = newHeight / 15 + 100;
    const maxScaleWidth = 1200;
    const minScaleWidth = 300;
    const scaleWidth = Math.max(Math.min(newWidth, maxScaleWidth), minScaleWidth);
    const fontSizeTitleComputed = 10 + scaleWidth / 20;
    return (
        <div
            style={{
                overflow: "hidden",
                position: "absolute",
                left: 0,
                top: 0,
                zIndex: -100,
                width: window.innerWidth - 20,
                height: window.innerHeight - 20,
            }}
        >
            <div
                style={{
                    position: "absolute",
                    left: 0,
                    top: 0,
                    zIndex: -100,
                    width: currentWidth.current,
                    height: currentHeight.current,
                }}
            >
                <div
                    className="welcome-box-title"
                    style={{ left: divX, top: divY, display: "flex", flexDirection: "column", position: "absolute" }}
                >
                    <div className="welcome-box-title" style={{ marginBottom: 10, fontSize: 34 }}>
                        Log in
                        <div style={{ marginBottom: 10, fontSize: fontSizeTitleComputed * 0.4 }}>
                            <div>{loginErrorMessage}</div>
                            <div>
                                <TextField
                                    slotProps={{
                                        inputLabel: {
                                            shrink: true,
                                        },
                                    }}
                                    disabled={loginInProgress}
                                    required
                                    label="email"
                                    autoComplete="email"
                                    margin="normal"
                                    value={currentLogin}
                                    onChange={(e) => setCurrentLogin(e.target.value)}
                                    style={{ width: 350 }}
                                ></TextField>
                            </div>
                            <div style={{ marginTop: 10 }}>
                                <TextField
                                    slotProps={{
                                        inputLabel: {
                                            shrink: true,
                                        },
                                    }}
                                    label={"Password"}
                                    disabled={loginInProgress}
                                    value={password}
                                    autoComplete="current-password"
                                    onChange={(e) => setPassword(e.target.value)}
                                    type={"password"}
                                    style={{ width: 350 }}
                                ></TextField>
                            </div>
                            <div style={{ marginTop: 20 }}>
                                <Button disabled={loginInProgress} onClick={() => loginAction()}>
                                    Login
                                </Button>
                                <Button disabled={loginInProgress} onClick={() => logout()}>
                                    Logout
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default LoginScreen;
