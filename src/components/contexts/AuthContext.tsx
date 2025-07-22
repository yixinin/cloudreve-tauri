// src/contexts/AuthContext.tsx
import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { LoginAck, User, userLogout } from '../../services/userService';
import { getStorageInfo } from '../../services/fileService';
import { useNavigate } from 'react-router-dom';


import { once } from '@tauri-apps/api/event';

type SessionChanged = {
    logined: boolean
};


type AuthContextType = {
    isAuthenticated: boolean;
    user: User | null;
    login: (ack: User) => void;
    logout: () => void;
    checkAuth: () => Promise<boolean>;
};

const AuthContext = createContext<AuthContextType>({
    isAuthenticated: false,
    user: null,
    login: () => { },
    logout: () => { },
    checkAuth: async () => false,
});


once<SessionChanged>('session-changed', (event) => {
    console.log("event:", event);

    const logout = () => {
        const navigate = useNavigate();
        localStorage.removeItem('userData');
        navigate("/login")
    }
    if (event.payload.logined === false) {
        logout()
    }
});

export const AuthProvider = ({ children }: { children: React.ReactNode }) => {
    const [isAuthenticated, setIsAuthenticated] = useState(false);
    const [user, setUser] = useState<User | null>(null);

    const navigate = useNavigate();
    // 初始化验证（含token有效性检查）
    const checkAuth = useCallback(async () => {
        const user = localStorage.getItem('userData');
        if (user) {
            const userData = JSON.parse(user);
            setUser(userData);
            try {
                const ack = await getStorageInfo();
                if (ack.total) {
                    setIsAuthenticated(true);
                    return true;
                } else {
                    setIsAuthenticated(false)
                }
            } catch (error) {
                setUser(null);
                setIsAuthenticated(false)
                console.error('Token验证失败:', error);
            }
        }

        return false;
    }, []);

    // 冷启动时验证token
    useEffect(() => {
        checkAuth();
    }, []);

    useEffect(() => {
        if (isAuthenticated) {
            navigate("/files")
        } else {
            navigate("/login")
        }
    }, [isAuthenticated])
    const login = (user: User) => {
        localStorage.setItem('userData', JSON.stringify(user));
        setUser(user);
        setIsAuthenticated(true);
    };

    const logout = async () => {
        if (isAuthenticated) {
            if (await userLogout()) {
                localStorage.removeItem('userData');
                setUser(null);
                setIsAuthenticated(false);
            }
        }
        navigate("/login")
    };

    return (
        <AuthContext.Provider
            value={{ isAuthenticated, user, login, logout, checkAuth }}
        >
            {children}
        </AuthContext.Provider>
    );
};

// 自定义Hook（推荐使用方式）
export const useAuth = () => {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error('useAuth必须在AuthProvider内使用');
    }
    return context;
};
