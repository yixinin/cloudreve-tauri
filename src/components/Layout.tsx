import React, { useState } from 'react';
import { Outlet } from 'react-router-dom';

import {
    Search as SearchIcon,
} from '@mui/icons-material';

import UserMenu from './menus/userMenu';
import AddMenu from './menus/addMenu';
import SideMenu from './menus/sideMenu';
import { Box, FormControl, IconButton, MenuItem, Select } from '@mui/material';
import { AppBarProvider, useAppBar } from './contexts/AppBarContext';
import FileNameDialog, { FileOp } from './pops/FileNameDialog';
import { createFolder } from '../services/fileService';
import SearchBox from './pops/SearchBox';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useNotification } from './contexts/NotificationProvider';
import { NetworkMode, setNetworkSettingsMode } from '../services/userService';

interface ProgressPayload {
    id: number,
    chunk: number,
    progress: number,
    total: number,
}
const Layout = () => {
    return (
        <AppBarProvider>
            <LayoutWraper />
        </AppBarProvider>
    );
};

const LayoutWraper: React.FC = () => {
    const { uploads, downloads, setUploads, setDownloads } = useAppBar();

    useEffect(() => {
        var unlistenDownload: UnlistenFn;
        var unlistenUpload: UnlistenFn;
        const listenTransferEvents = async () => {
            unlistenDownload = await listen<ProgressPayload>('download://progress', (event) => {
                // console.log("download event", event);

                const { id, progress, total } = event.payload;
                setDownloads(downloads.map(item => {
                    if (item.id !== id) {
                        return item
                    }
                    console.log("download event", id, progress, total);
                    return {
                        ...item,
                        progress: progress,
                        size: total,
                        status: progress >= total ? 'completed' : 'downloading'
                    }
                }));
            });

            unlistenUpload = await listen<ProgressPayload>('upload://progress', (event) => {
                // console.log("upload event", event);
                const { id, progress, total } = event.payload;
                setUploads(uploads.map(item => {
                    if (item.id !== id) {
                        return item
                    }
                    ;
                    console.log("upload event", id, progress, total);
                    return {
                        ...item,
                        progress: progress,
                        size: total,
                        status: progress >= total ? 'completed' : 'uploading'
                    }
                }));
            });
        }

        listenTransferEvents();

        return () => {
            if (unlistenDownload) {
                unlistenDownload();
            }
            if (unlistenUpload) {
                unlistenUpload();
            }
        };
    }, [])
    const { filePageProps, setSearchProps, siteAddrs, networkMode, setNetworkMode } = useAppBar();
    const [isNewFolerOpen, setIsNewFolerOpen] = useState(false);
    const [isSerchBoxOpen, setIsSerchBoxOpen] = useState(false);

    const { notify } = useNotification();

    const handleNewFolder = () => {
        setIsNewFolerOpen(true)
    }
    const handleNewFolderClose = () => {
        setIsNewFolerOpen(false)
    }

    const handleNewFolderSubmit = async (newName: string) => {
        const uri = `${filePageProps.currentPath}/${newName}`;
        try {
            await createFolder(uri);
            notify(`${newName}已创建`)
        }
        catch (err) {
            console.log("operation error:", err);
            notify(`创建失败:${err}`, 'error')
        }
    }

    const handleSerchBoxClose = () => {
        setIsSerchBoxOpen(false);
    }

    const handleSearchClick = () => {
        setIsSerchBoxOpen(true);
    }

    const handleSearch = (name: string) => {
        setSearchProps({ name: name })
    }
    const handleNetworkModeChange = async (mode: NetworkMode) => {
        setNetworkMode(mode);
        await setNetworkSettingsMode(mode)
    }

    return (
        <Box sx={{
            height: '100vh',
            width: '100%',
            overflow: 'hidden',
            scale: 'none',
            backgroundColor: '#rgb(254,254,254)',
        }}>
            <Box sx={{
                width: '100%',
                display: 'flex',
                justifyContent: 'space-between',
                backgroundColor: 'transparent'
            }}>
                <Box sx={{
                    marginLeft: 2,

                }}>
                    <SideMenu />
                </Box>


                <Box sx={{
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'center'
                }}>
                    <FormControl fullWidth>
                        <Select
                            id="demo-simple-select"
                            value={networkMode}
                            onChange={(e) => {
                                handleNetworkModeChange(e.target.value)
                            }}
                            size='small'
                            sx={{
                                border: 0,
                                borderColor: 'transparent'
                            }}
                        >
                            {siteAddrs.map(item => (
                                <MenuItem value={item.mode}>{item.addr}</MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                    <IconButton onClick={handleSearchClick} >
                        <SearchIcon />
                    </IconButton>

                    <AddMenu
                        handleNewFloder={handleNewFolder}
                        currentPath={filePageProps.currentPath}
                    />

                    <IconButton>
                        <UserMenu />
                    </IconButton>
                </Box>
            </Box>

            <Box sx={{
                height: '100%',
                width: '100%',
                display: 'flex',
                flexDirection: 'column'
            }}>
                <Box
                    component="main"
                    sx={{
                        flex: 1,
                        flexGrow: 1,
                        p: 1,
                        width: '100%',
                        backgroundColor: 'rgb(245,245,245)',
                        overflow: 'hidden'
                    }}
                >
                    <Outlet />

                    <FileNameDialog
                        open={isNewFolerOpen}
                        op={FileOp.create}
                        targetName={'新建文件夹'}
                        fileType={1}
                        onClose={handleNewFolderClose}
                        onSubmit={handleNewFolderSubmit}
                    />

                    <SearchBox
                        open={isSerchBoxOpen}
                        onClose={handleSerchBoxClose}
                        onSearch={handleSearch}
                    />
                </Box>
            </Box>

        </Box >
    );
};

export default Layout;