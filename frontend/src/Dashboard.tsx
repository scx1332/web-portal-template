import React, { useContext } from "react";

import { Routes, Route, Link } from "react-router-dom";
import LoginScreen from "./LoginScreen";
import { LoginStatus, useLogin, useLoginEvent, useLoginOrNull } from "./LoginProvider";
import WelcomeScreen from "./WelcomeScreen";
import { Button } from "@mui/material";
import { backendFetch } from "./common/BackendCall";
import Blocks from "./Blocks";

const Dashboard = () => {
    const loginInformation = useLoginOrNull();

    const [logoutInProgress, setLogoutInProgress] = React.useState(false);
    const updateLogin = useLoginEvent();
    const logout = async () => {
        setLogoutInProgress(true);
        const response = await backendFetch("/api/logout", {
            method: "Post",
        });
        const data = await response.text();
        if (data === "Logged out") {
            updateLogin({
                loggedIn: null,
                loggedOut: true,
            });
        }

        setLogoutInProgress(false);
    };

    if (loginInformation.loginData == null && loginInformation.loginStatus === LoginStatus.LOGIN_IN_PROGRESS) {
        return <div>{loginInformation.loginError}</div>;
    }
    return (
        <div className="main-page">
            <div className="top-header">
                <div className="top-header-title">Web Portal</div>
                <div className="top-header-navigation">
                    <Link to="/">Main</Link>
                    <Link to="/blocks">Blocks</Link>

                    {loginInformation.loginData ? (
                        <div>
                            {loginInformation.loginData.email}
                            <Button onClick={(e) => logout()}>Logout</Button>
                        </div>
                    ) : (
                        <Link to="/login">Login</Link>
                    )}
                </div>
            </div>
            <div className="main-content">
                <Routes>
                    <Route
                        path="/"
                        element={
                            <div>
                                <WelcomeScreen />
                            </div>
                        }
                    />
                    <Route
                        path="/login"
                        element={
                            <div>
                                <LoginScreen />
                            </div>
                        }
                    />
                    <Route
                        path="/blocks"
                        element={
                            <div>
                                <Blocks />
                            </div>
                        }
                    />
                </Routes>
            </div>
        </div>
    );
};

export default Dashboard;
