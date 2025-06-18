
import React, { createContext, useContext, useEffect, useState } from 'react';

import {
    Home as HomeIcon,
    Photo as PhotoIcon,
    Movie as MovieIcon,
    AudioFile as AudioIcon,
    EditDocument as DocumentIcon,
    Share as ShareIcon,
    Recycling as TrashIcon,
} from '@mui/icons-material';
import { DownloadTask, UploadTask } from '../../services/upload';
import { getNetworkSetting, NetworkMode } from '../../services/userService';

interface AppBarContextType {
    filePageProps: FilePageProps;
    setFilePageProps: (props: FilePageProps) => void;
    searchProps: searchProps;
    setSearchProps: (props: searchProps) => void;
    downloads: DownloadTask[];
    setDownloads: (tasks: DownloadTask[]) => void;
    uploads: UploadTask[];
    setUploads: (tasks: UploadTask[]) => void;
    networkMode: NetworkMode,
    setNetworkMode: (mode: NetworkMode) => void;
    siteAddrs: Addr[],
}
export const HomePath = 'cloudreve://my'

export interface FilePageProps {
    currentPath: string; // /
    category: string; // image/video
    rootPath: string; // my/share ...
    name: string; // 我的文件/图片/视频
    icon: React.ReactNode;
}

export interface searchProps {
    name: string;
}



export interface Addr {
    addr: string,
    mode: NetworkMode,
}
export const pagesItems: FilePageProps[] = [
    { currentPath: HomePath, name: '我的文件', icon: <HomeIcon />, rootPath: HomePath, category: '' },
    { currentPath: HomePath, name: '图片', icon: <PhotoIcon />, rootPath: HomePath, category: 'image' },
    { currentPath: HomePath, name: '视频', icon: <MovieIcon />, rootPath: HomePath, category: 'video' },
    { currentPath: HomePath, name: '音乐', icon: <AudioIcon />, rootPath: HomePath, category: 'audio' },
    { currentPath: HomePath, name: '文档', icon: <DocumentIcon />, rootPath: HomePath, category: 'document' },
    { currentPath: 'cloudreve://shared_with_me', name: '与我共享', icon: <ShareIcon />, rootPath: 'cloudreve://shared_with_me', category: '' },
    { currentPath: 'cloudreve://trash', name: '回收站', icon: <TrashIcon />, rootPath: 'cloudreve://trash', category: '' },
];


const AppBarContext = createContext<AppBarContextType | undefined>(undefined);

export const AppBarProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [filePageProps, setFilePageProps] = useState<FilePageProps>(pagesItems[0]);
    const [searchProps, setSearchProps] = useState<searchProps>({ name: '' })
    const [downloads, setDownloads] = useState<DownloadTask[]>([]);
    const [uploads, setUploads] = useState<UploadTask[]>([]);
    const [networkMode, setNetworkMode] = useState<NetworkMode>(NetworkMode.Auto);
    const [siteAddrs, setSiteAddrs] = useState<Addr[]>([]);


    useEffect(() => {
        var loadSite = async () => {
            const setting = await getNetworkSetting();
            setNetworkMode(setting.mode);
            var addrs: Addr[] = [{
                mode: NetworkMode.Auto,
                addr: "auto",
            }];
            if (setting.addr) {
                addrs.push({
                    mode: NetworkMode.Normal,
                    addr: setting.addr,
                })
            }
            if (setting.addr6) {
                addrs.push({
                    mode: NetworkMode.IPv6,
                    addr: setting.addr6,
                })
            }
            addrs.push({
                mode: NetworkMode.P2P,
                addr: "p2p",
            })
            setSiteAddrs(addrs);
        }
        loadSite();
    }, [])
    return (
        <AppBarContext.Provider value={{
            filePageProps, setFilePageProps,
            searchProps, setSearchProps,
            downloads, setDownloads,
            uploads, setUploads,
            networkMode, setNetworkMode,
            siteAddrs,
        }}>
            {children}
        </AppBarContext.Provider>
    );
};

export const useAppBar = () => {
    const context = useContext(AppBarContext);
    if (!context) throw new Error('useAppBar必须在AppBarProvider内使用');
    return context;
};