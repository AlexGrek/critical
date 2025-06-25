import type { AuthContextType, UserWhoami } from "./AuthProvider";
import { authFetchWithStatus } from "../../utils";

const ENDPOINT = '/api/v1/auth/whoami';

export const fetchUserWhoami = async (auth: AuthContextType) => {
    const { value, statusCode } = await authFetchWithStatus<UserWhoami>(ENDPOINT)
    if (statusCode == 401) {
        // we are not logged in
        auth.logout()
        console.log("Logging out, token is invalid")
    } else if (statusCode >= 400) {
        console.error("Server returned ", statusCode)
        auth.logout()
    } else {
        if (value) {
            auth.setUserInfo(value)
        } else {
            console.warn("AUTH: No data")
        }
    }
};

export const fetchInitialWhoami = async () => {
    const token = localStorage.getItem('authToken');
    if (!token) {
        console.log("we have no token, so not logging in");
        return;
    }
    const { value, statusCode } = await authFetchWithStatus<UserWhoami>(ENDPOINT)
    if (statusCode == 401) {
        console.log("Logging out, token is invalid")
        // we are not logged in
        return null;
    } else if (statusCode >= 400) {
        console.error("Server returned ", statusCode)
        return null;
    } else {
        if (value) {
            console.log(value);
            return value;
        } else {
            console.error("No data in whoami request")
            return null;
        }
    }
}