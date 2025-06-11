// src/contexts/NotificationContext.tsx
import React, { createContext, useContext, useState, ReactNode } from 'react';
import { Alert, AlertColor, Snackbar } from '@mui/material';

interface Notification {
    message: string;
    severity: AlertColor;
}

interface NotificationContextType {
    notify: (message: string, severity?: AlertColor) => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export const NotificationProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [notification, setNotification] = useState<Notification | null>(null);
    const [open, setOpen] = useState(false);

    const notify = (message: string, severity: AlertColor = 'success') => {
        setNotification({ message, severity });
        setOpen(true);
    };

    const handleClose = () => {
        setOpen(false);
    };

    return (
        <NotificationContext.Provider value={{ notify }}>
            {children}
            <Snackbar
                open={open}
                autoHideDuration={6000}
                onClose={handleClose}
                anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
            >
                <Alert
                    onClose={handleClose}
                    severity={notification?.severity || 'error'}
                    sx={{ width: '100%' }}
                >
                    {notification?.message}
                </Alert>
            </Snackbar>
        </NotificationContext.Provider>
    );
};

export const useNotification = () => {
    const context = useContext(NotificationContext);
    if (!context) {
        throw new Error('useNotification must be used within a NotificationProvider');
    }
    return context;
};