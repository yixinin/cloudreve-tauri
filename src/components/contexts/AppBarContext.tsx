
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
import { getLoginData, getUserSetting } from '../../services/userService';

interface AppBarContextType {
    filePageProps: FilePageProps;
    setFilePageProps: (props: FilePageProps) => void;
    searchProps: searchProps;
    setSearchProps: (props: searchProps) => void;
    downloads: DownloadTask[];
    setDownloads: (tasks: DownloadTask[]) => void;
    uploads: UploadTask[];
    setUploads: (tasks: UploadTask[]) => void;
    siteAddr: string,
    setSiteAddr: (addr: string) => void;
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
    kind: string,
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
    const [siteAddr, setSiteAddr] = useState<string>("");
    const [siteAddrs, setSiteAddrs] = useState<Addr[]>([]);


    useEffect(() => {
        var loadSite = async () => {
            const loginData = await getLoginData();
            console.log('login data: ', loginData);

            var addrs = [
                { addr: '自动选择', kind: 'auto' },
                { addr: loginData.addr, kind: 'ipv4' }]
            if (loginData.ipv6 && loginData.addr6) {
                addrs.push({
                    addr: loginData.addr6,
                    kind: 'ipv6',
                })
            }

            setSiteAddrs(addrs)

            const setting = await getUserSetting();
            if (setting.force_ipv6 && loginData.addr6) {
                setSiteAddr('ipv6')
            } else if (setting.force_ipv4 && loginData.addr) {
                setSiteAddr('ipv4')
            } else {
                setSiteAddr('auto')
            }
        }
        loadSite();
    }, [])
    return (
        <AppBarContext.Provider value={{
            filePageProps, setFilePageProps,
            searchProps, setSearchProps,
            downloads, setDownloads,
            uploads, setUploads,
            siteAddr, setSiteAddr,
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