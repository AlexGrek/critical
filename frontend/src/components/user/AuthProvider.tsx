
import { createContext, type ReactNode, useContext, useEffect, useState } from 'react';
import { fetchInitialWhoami } from './login_utils';

export type AuthContextType = {
  getIsAnonymous: () => boolean;
  getCurrentUserId: () => string | null;
  setUserInfo: (value: UserWhoami | null) => void;
  getUserInfo: () => UserWhoami | null;
  user: UserWhoami | null;
  logout: () => void;
};

export type UserWhoami = {
  uid: string;
  email: string;
  hashed_password: string;
  role: string;
  metadata: Record<string, unknown>; // Use Record for flexible metadata
  created_at: string;
  updated_at: string;
}

const defaultAuthContext: AuthContextType = {
  getCurrentUserId: () => null,
  getUserInfo: () => null,
  getIsAnonymous: () => false,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  setUserInfo: (_: UserWhoami | null) => { },
  logout: () => { },
  user: null
};

const AuthContext = createContext<AuthContextType>(defaultAuthContext);

// Create a provider component
export const AuthProvider = ({ children }: { children: ReactNode }) => {
  const [userInfo, setUserInfo] = useState<UserWhoami | null>(null);

  useEffect(() => {
    const updateUserInfoAsync = async () => {
      const info = await fetchInitialWhoami();
      if (info == null) {
        return;
      }
      setUserInfo(info);
    }
    updateUserInfoAsync();
  }, [])

  const getUserInfo = () => {
    console.log("User info is ", userInfo);
    return userInfo
  }

  const getIsAnonymous = () => {
    if (userInfo != null) {
      const value = userInfo.metadata["anonymous"];
      return value ? true : false;
    }
    return false
  }

  const getCurrentUserId = () => {
    if (userInfo != null) {
      return userInfo.uid
    }
    return null
  }

  const logout = () => {
    setUserInfo(null)
  }

  return (
    <AuthContext.Provider value={{ getUserInfo, getIsAnonymous, getCurrentUserId, setUserInfo, logout, user: userInfo }}>
      {children}
    </AuthContext.Provider>
  );
};

// eslint-disable-next-line react-refresh/only-export-components
export const useAuth = () => useContext(AuthContext);