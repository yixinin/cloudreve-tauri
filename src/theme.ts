import { createTheme } from '@mui/material/styles';

export const androidTheme = createTheme({
    palette: {
        primary: { main: '#1976d2' }, // 安卓蓝
        secondary: { main: '#4caf50' } // 安卓绿
    },
    components: {
        MuiListItemButton: {
            styleOverrides: {
                root: {
                    '&:active': { // 安卓点击效果
                        backgroundColor: 'rgba(25, 118, 210, 0.1)'
                    }
                }
            }
        },
        MuiDrawer: {
            styleOverrides: {
                paper: {
                    scrollbarWidth: 'thin', // 安卓滚动条
                    '&::-webkit-scrollbar': { width: 6 },
                    '&::-webkit-scrollbar-thumb': { borderRadius: 3 }
                }
            }
        }
    }
});