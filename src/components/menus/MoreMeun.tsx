import {
    Autorenew as RefreshIcon,
    MoreHoriz as MoreIcon,
} from '@mui/icons-material';
import { IconButton, ListItemIcon, ListItemText, Menu, MenuItem } from '@mui/material';
import { useState } from 'react';

interface MoreMenuProps {
    handleRefresh: () => void;
}


const MoreMenu: React.FC<MoreMenuProps> = ({
    handleRefresh
}) => {

    const [anchorEl, setAnchorEl] = useState(null);
    const isOpen = Boolean(anchorEl);
    // 打开菜单
    const handleClick = (event: any) => {
        setAnchorEl(event.currentTarget);
    };

    // 关闭菜单
    const handleClose = () => {
        setAnchorEl(null);
    };

    const handleRefreshClick = () => {
        handleRefresh()
        handleClose()
    }

    return (
        <>
            <IconButton onClick={handleClick} sx={{
                width: 45,
                borderRadius: '0 10px 10px 0'
            }}>
                <MoreIcon />
            </IconButton >



            {/* 菜单内容 */}
            <Menu
                anchorEl={anchorEl}
                open={isOpen}
                onClose={handleClose}

                transformOrigin={{
                    vertical: "top",
                    horizontal: "right",
                }}
                anchorOrigin={{
                    vertical: "bottom",
                    horizontal: "right",
                }}>
                <MenuItem onClick={handleRefreshClick}>
                    <ListItemIcon>
                        <RefreshIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText primary="刷新" />
                </MenuItem>
            </Menu >
        </>
    );

}

export default MoreMenu;