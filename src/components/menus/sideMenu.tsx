import { useState } from "react";
import {
    Menu,
    ListItemIcon,
    ListItemText,
    IconButton,
    Divider,
} from "@mui/material";

import MenuItem from '@mui/material/MenuItem';
import {
    Menu as MenuIcon,
    Image as ImageIcon,
    ImportExport,
    Share,
} from '@mui/icons-material';
import { useNavigate } from "react-router-dom";
import { FilePageProps, useAppBar, pagesItems } from "../contexts/AppBarContext";




const SideMenu = () => {
    const [anchorEl, setAnchorEl] = useState(null);
    const isOpen = Boolean(anchorEl);

    const { setFilePageProps } = useAppBar();

    // 打开菜单
    const handleClick = (event: any) => {
        setAnchorEl(event.currentTarget);
    };

    // 关闭菜单
    const handleClose = () => {
        setAnchorEl(null);
    };

    const handleItemClick = (item: FilePageProps) => {
        if (item) {
            navigate("/files")
            setFilePageProps(item);
        }
        handleClose()
    }
    const navigate = useNavigate();
    const handleTransfers = () => {
        navigate("/transfers")
    }
    const handleShares = () => {
        navigate("/shares")
    }
    const handleAlbum = () => {
        window.location.href = '/albums'
    }

    return (
        <>
            <IconButton
                aria-label="open drawer"
                edge="start"
                sx={{ mr: 2 }}
                onClick={handleClick}
            >
                <MenuIcon />
            </IconButton>

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
                }}
                disableScrollLock={true}
            >

                {pagesItems.map((item) => (

                    < MenuItem key={item.name} onClick={() => handleItemClick({
                        ...item,
                        currentPath: item.rootPath,
                    })}>
                        <ListItemIcon>
                            {item.icon}
                        </ListItemIcon>
                        <ListItemText primary={item.name} />
                    </MenuItem>
                ))}

                <Divider />
                <MenuItem key='/shares' onClick={handleShares}>
                    <ListItemIcon>
                        <Share />
                    </ListItemIcon>
                    <ListItemText primary="我的分享" />
                </MenuItem>
                <Divider />

                <MenuItem key='transfers' onClick={handleTransfers}>
                    <ListItemIcon>
                        <ImportExport />
                    </ListItemIcon>
                    <ListItemText primary="上传/下载" />
                </MenuItem>

                {false && (
                    <MenuItem key='albums' onClick={handleAlbum}>
                        <ListItemIcon>
                            <ImageIcon />
                        </ListItemIcon>
                        <ListItemText primary="本地相册" />
                    </MenuItem>
                )}

            </Menu >
        </>
    );
};

export default SideMenu;