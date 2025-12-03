import { useEffect, useState } from "react";
import {
    Avatar,
    Menu,
    MenuItem,
    ListItemIcon,
    ListItemText,
    Typography,
    Divider,
    Box,
} from "@mui/material";
import {
    Person as PersonIcon,
    Home as HomeIcon,
    ExitToApp as LogoutIcon,
} from "@mui/icons-material";
import { User, userLogout } from "../../services/userService";
import { useAuth } from "../contexts/AuthContext";
import { Settings as SettingsIcon } from '@mui/icons-material';

const UserMenu = () => {
    const [anchorEl, setAnchorEl] = useState(null);
    const open = Boolean(anchorEl);

    const [user, setUser] = useState({
        nickname: 'user',
        email: '',
        group: { name: '' },
    })

    useEffect(() => {
        const userData = localStorage.getItem('userData');
        if (userData) {
            const user: User = JSON.parse(userData);
            if (user) {
                setUser(user)
            }
        }
    }, [])



    const { logout } = useAuth();

    // 打开菜单
    const handleClick = (event: any) => {
        setAnchorEl(event.currentTarget);
    };

    // 关闭菜单
    const handleClose = () => {
        setAnchorEl(null);
    };

    // 退出登录
    const handleLogout = async () => {
        userLogout().then((_) => {
            console.log("logout success");
        }).catch((err) => {
            console.log("logout failed", err);
        }).finally(() => {
            logout()
            handleClose();
        })
    };

    return (
        <>
            {/* 点击头像触发菜单 */}
            <Avatar
                src={user.nickname}
                alt={user.nickname}
                onClick={handleClick}
                sx={{ cursor: "pointer", width: 30, height: 30, padding: 0 }} // 调整头像大小
            />

            {/* 菜单内容 */}
            <Menu
                anchorEl={anchorEl}
                open={open}
                onClose={handleClose}

                transformOrigin={{
                    vertical: "top",
                    horizontal: "right",
                }}
                anchorOrigin={{
                    vertical: "bottom",
                    horizontal: "right",
                }}
            >
                {/* 1. 用户信息 */}
                <MenuItem onClick={handleClose} sx={{ cursor: "default" }}>
                    <ListItemIcon>
                        <PersonIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText
                        primary={
                            <Box sx={{
                                display: 'flex'
                            }}>
                                <Typography fontWeight="bold">{user.nickname}</Typography>
                                <Typography sx={{
                                    marginLeft: 1
                                }}>{user.group.name}</Typography>
                            </Box>

                        }
                        secondary={user.email}
                    />
                </MenuItem>

                <Divider sx={{ my: 1 }} />

                {/* 2. 设置 */}
                <MenuItem onClick={handleSettings}>                    <ListItemIcon>
                    <SettingsIcon fontSize="small" />
                </ListItemIcon>
                    <ListItemText primary="设置" />
                </MenuItem>

                {/* 3. 个人主页 */}
                <MenuItem onClick={handleClose}>
                    <ListItemIcon>
                        <HomeIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText primary="个人主页" />
                </MenuItem>

                {/* 3. 退出登录 */}
                <MenuItem onClick={handleLogout}>
                    <ListItemIcon>
                        <LogoutIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText primary="退出登录" />
                </MenuItem>
            </Menu>
        </>
    );
};

export default UserMenu;


const handleSettings = () => {
    navigate('/settings');
    handleClose();
};