import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function useAndroidStatusBar() {
    useEffect(() => {
        if (/Android/.test(navigator.userAgent)) {
            invoke('set_status_bar_color', { color: '#1976d2' });
            invoke('set_navigation_bar_color', { color: '#ffffff' });
        }
    }, []);
}