import React, { useState, useEffect } from 'react';
import {
    Box,
    Typography,
    Paper,
    IconButton,
    Avatar,
    Stack,
    Divider
} from "@mui/material";
import NavigateNextIcon from '@mui/icons-material/NavigateNext';
import {
    Home as HomeIcon,
    Autorenew as RefreshIcon,
    ViewList as ListIcon,
    Collections as GalleryIcon,
    Apps as GridIcon,

} from '@mui/icons-material';

import {
    onBackKeyDown,
} from "tauri-plugin-app-events-api";


import { deleteFile, FileItem, getFileListByCategory, getThumbURL, getURL, preDownloadFile, renameFile, restoreFile, searchFilesByName, unlockFile } from '../../services/fileService';
import { useNavigate } from 'react-router-dom';
import ImagePreview from '../../components/pops/ImagePreview';

import { HomePath, useAppBar } from '../contexts/AppBarContext';
import FileListView from './views/FileListView';
import { FileKind, getDefaultThumb, getFileKind } from '../../utils/file';
import FileGridView from './views/FileGridView';
import FileGallaryView from './views/FileGallaryView';
import FileContextMenu, { position } from '../menus/FileContextMenu';
import FileNameDialog, { FileOp } from '../pops/FileNameDialog';
import FileDeleteDialog from '../pops/FileDeleteDialog';
import FilePreviewDialog from '../pops/FileDetailsDialog';
import MoreMenu from '../menus/MoreMeun';
import ViewToggle, { ViewMode } from '../pops/SwitchDialog';
import SortMenu, { Direction, Sort } from '../menus/SortMenu';
import ShareDialog from '../pops/ShareDialog';
import LinkDialog from '../pops/LinkDialog';
import { downloadFile, DownloadTask } from '../../services/upload';
import DirectorySelector from '../pops/DirectorySelector';
import { useNotification } from '../contexts/NotificationProvider';
import { FileTag } from '../../services/fileModel';
import { CodeError } from '../../services/proto';
import { useAuth } from '../contexts/AuthContext';


// 面包屑项类型
interface BreadcrumbItem {
    name: string;
    path: string;
}






const searchFiles = async (name: string, page: number, rowsPerPage: number, orderBy: keyof FileItem, order: string): Promise<{
    files: FileItem[];
    file_tags: FileTag[][];
    tags: string[];
    total: number;
}> => {
    const ack = await searchFilesByName(name, HomePath, page, rowsPerPage, orderBy, order)
    return {
        files: ack.files,
        file_tags: ack.file_tags,
        tags: ack.tags,
        total: ack.pagination.page_size,
    };
};



const FileListPage: React.FC = () => {
    const { filePageProps, setFilePageProps, searchProps, setSearchProps } = useAppBar();

    const [currentPath, setCurrentPath] = useState(filePageProps.currentPath);
    // 面包屑导航
    const [breadcrumbs, setBreadcrumbs] = useState<BreadcrumbItem[]>([{ name: 'Home', path: '/' }]);
    // 文件列表
    const [files, setFiles] = useState<FileItem[]>([]);
    const [_, setTags] = useState<string[]>([])
    // 分页
    const [page, setPage] = useState(0);
    const [rowsPerPage] = useState(50);
    // 展示模式
    const [viewMode, setViewMode] = useState<ViewMode>('grid');
    const [orderBy, setOrderBy] = useState<Sort>("created_at");
    const [order, setOrder] = useState<Direction>("desc");
    const [previewImage, setPreviewImage] = useState<{
        open: boolean;
        url: string;
        name: string;
    }>({
        open: false,
        url: '',
        name: ''
    });

    const { logout } = useAuth();

    const onSrotChange = (sort: Sort, direction: Direction) => {
        setOrder(direction);
        setOrderBy(sort)
    }

    const [loadingThumbnails, setLoadingThumbnails] = useState<Record<string, boolean>>({});

    const [fileMenuPos, setfileMenuPos] = useState<position | null>(null);

    const [fileAction, setFileAction] = useState('');

    const [refresh, setRefresh] = useState('')

    const [isViewModeOpen, setIsViewModeOpen] = useState(false);
    const [viewModePos, setViewModePos] = useState<position | null>(null);
    const [showThumb, setShowThumb] = useState(true);
    const [selectedFile, setselectedFile] = useState<FileItem | null>(null);

    onBackKeyDown(() => {
        if (fileAction !== '') {
            setFileAction('')
            return
        }

        const props = filePageProps;
        if (props.currentPath === props.rootPath) {
            if (props.category !== '') {
                setFilePageProps({
                    ...props,
                    category: ''
                });
                return
            }
        } else {
            const idx = props.currentPath.lastIndexOf('/');
            if (idx >= 0) {
                let parent = props.currentPath.slice(0, idx) + '/'
                console.log("on back key donw, current: ", props.currentPath, "cate: ", props.category, "parent: ", parent);
                handlePathChange(parent);
            }
        }
    })

    const navigate = useNavigate();
    const loadFiles = async () => {
        if (searchProps.name !== '') {
            const { files, tags, file_tags } = await searchFiles(searchProps.name, page, rowsPerPage, orderBy, order);
            const tagfiles = files.map((item, i) => {
                return {
                    ...item,
                    tags: file_tags[i],
                }
            })
            setFiles(tagfiles);
            setTags(tags)
        } else {
            try {
                const ack = await getFileListByCategory(filePageProps.currentPath, filePageProps.category, page, rowsPerPage, orderBy, order)
                if (ack.storage_policy) {
                    localStorage.setItem('policyId', ack.storage_policy.id);
                }
                const tagfiles = ack.files.map((item, i) => {
                    return {
                        ...item,
                        tags: ack.file_tags[i],
                    }
                })
                setFiles(tagfiles);
                setTags(ack.tags)
            }
            catch (err) {
                const { code, msg } = err as CodeError;
                if (code == 401) {
                    logout()
                    return
                } else {
                    console.log(code, msg);
                }
            }
        }
    };

    useEffect(() => {
        if (fileAction === '' && selectedFile && refresh !== '') {
            loadFiles()
            setRefresh('')
            return
        }

        if (fileAction !== '') {
            setfileMenuPos(null)
        }
        if (selectedFile) {
            switch (fileAction) {
                case 'open':
                    onFileClick(selectedFile)
                    break
                case 'download':
                    handleDownload(selectedFile)
                    break
                case 'restore':
                    handleRestore(selectedFile)
            }
        }


    }, [fileAction])

    useEffect(() => {
        console.log("loading files", currentPath, order, orderBy, 'search', searchProps.name);
        loadFiles();
    }, [currentPath, filePageProps, searchProps, order, orderBy])
    // 加载缩略图
    useEffect(() => {
        if (viewMode === 'grid' || viewMode === 'gallery') {
            const loadThumbnails = async () => {
                const filesToLoad = files.filter(
                    file => {
                        const fileKind = getFileKind(file.type, file.name);
                        if (fileKind == FileKind.image || fileKind == FileKind.video) {
                            if (file.type === 0 && !file.thumbnailUrl && !loadingThumbnails[file.id]) {
                                return true
                            }
                        }
                        return false
                    }
                );

                const filesToDefault = files.filter(
                    file => {
                        const fileKind = getFileKind(file.type, file.name);
                        if (fileKind == FileKind.image || fileKind == FileKind.video) {
                            return false
                        }
                        if (file.thumbnailUrl) {
                            return false
                        }
                        return true
                    }
                );

                if (filesToDefault.length > 0) {
                    filesToDefault.map((file) => {
                        setFiles(prevFiles =>
                            prevFiles.map(f =>
                                f.id === file.id ? { ...f, thumbnailUrl: getDefaultThumb(file.type, file.path) } : f
                            )
                        );
                    })
                }

                if (filesToLoad.length > 0) {
                    console.log(filesToLoad.length, " files to load thumbnails");
                    setLoadingThumbnails(prev => ({
                        ...prev,
                        ...filesToLoad.reduce((acc, file) => ({ ...acc, [file.id]: true }), {})
                    }));

                    await Promise.all(
                        filesToLoad.map(async (file) => {
                            try {
                                const thumbnailUrl = await getThumbURL(file.path);
                                console.log(file.id, thumbnailUrl);

                                setFiles(prevFiles =>
                                    prevFiles.map(f =>
                                        f.id === file.id ? { ...f, thumbnailUrl } : f
                                    )
                                );
                            } catch (error) {
                                setFiles(prevFiles =>
                                    prevFiles.map(f =>
                                        f.id === file.id ? { ...f, thumbnailUrl: getDefaultThumb(file.type, file.path) } : f
                                    )
                                );
                                console.error(`Failed to load thumbnail for ${file.name}:`, error);
                            } finally {
                                setLoadingThumbnails(prev => ({ ...prev, [file.id]: false }));
                            }
                        })
                    );
                }
            };

            loadThumbnails();
        }
    }, [files, viewMode]);

    // 处理路径变更
    const handlePathChange = (path: string) => {
        console.log("handle page changed", path);
        if (path) {
            const props = filePageProps;
            setFilePageProps({
                ...props,
                currentPath: path
            });
            setSearchProps({ name: '' });
            setCurrentPath(path);

            // 更新面包屑 
            const parts = path.replace(props.rootPath, '').split('/').filter(p => p);
            const newBreadcrumbs = parts.map((part, index) => ({
                name: part,
                path: props.rootPath + '/' + parts.slice(0, index + 1).join('/')
            }));

            setBreadcrumbs([{ name: 'Home', path: props.rootPath }, ...newBreadcrumbs]);
            console.log(breadcrumbs);
        }
        // 重置分页
        setPage(0);
    };

    const handleRefreshFiles = () => {
        loadFiles();
    }

    // 处理分页变更
    // const handleChangePage = (event: unknown, newPage: number) => {
    //     setPage(newPage);
    // };

    // 处理展示模式变更
    const handleViewModeChange = (
        newViewMode: ViewMode | null,
    ) => {
        if (newViewMode !== null) {
            setViewMode(newViewMode);
        }
    };

    const handleShowThumbChange = (show: boolean) => {
        setShowThumb(show)
    }


    const handleFolderClick = (filePath: string) => {
        if (filePageProps.currentPath) {
            handlePathChange(decodeURIComponent(filePath));
        }
    };

    const handleImagePreview = (url: string, name: string) => {
        console.log("set image url", url, name);

        setPreviewImage({
            open: true,
            url,
            name
        });
    };


    const handleClosePreview = () => {
        setPreviewImage(_prev => ({ url: '', name: '', open: false }));
        setFileAction('')
    };

    const handleVideoPlay = (url: string, name: string) => {
        navigate("/player/video?" + 'url=' + encodeURIComponent(url) + '&name=' + name)
    }
    const { downloads, setDownloads } = useAppBar();
    const { notify } = useNotification()
    const handleDownload = async (file: FileItem) => {
        const now = Math.floor(Date.now() / 1000)

        try {
            const { id, url, file_path } = await preDownloadFile(file.path, file.name)
            var task = {
                id: id,
                filename: file.name,
                fullPath: file_path,
                url: url,
                status: 'pending',
                size: 0,
                startTime: now,
                progress: 0,
            } as DownloadTask;
            const list = downloads;
            list.push(task);
            setDownloads(list);
            notify(file.name + " 开始下载")
            await downloadFile(id, url, file_path,)
        }
        catch (err) {
            notify("下载文件失败 " + err)
            console.log("download fail, error", err);
        }
    }

    const handleRestore = async (file: FileItem) => {
        try {
            await restoreFile(file.path);
        } catch (err) {
            console.log("restore file error:", err);

        }
    }


    const onFileClick = (file: FileItem) => {
        switch (getFileKind(file.type, file.name)) {
            case FileKind.folder:
                handleFolderClick(file.path)
                break
            case FileKind.image:
                handleImagePreview(file.path, file.name)
                break
            case FileKind.video:
                handleVideoPlay(file.path, file.name);
                break
            case FileKind.audio:
            case FileKind.document:
        }
    }

    const onFileRightClick = async (event: any, file: FileItem) => {
        setfileMenuPos({
            x: event?.clientX,
            y: event?.clientY,
        })
        setselectedFile(file);
    }

    const handleViewModeClick = (event: any) => {
        setIsViewModeOpen(!isViewModeOpen)
        const pos = { x: event?.clientX, y: event?.clientY };
        setViewModePos(pos)
    }

    const handleRenameFile = async (newName: string) => {
        if (fileAction === 'rename' && selectedFile) {
            try {
                await renameFile(newName, selectedFile.path)
                setRefresh('rename')
            }
            catch (err) {
                console.log("rename file error", err);
            }
        }
    }
    const handleDeleteFile = async (force: boolean) => {
        if (fileAction === 'delete' && selectedFile) {
            try {
                const ack = await deleteFile(selectedFile.path, false, !force)
                if (ack.token) {
                    // setFileAction('unlock')
                    if (force) {
                        await handleUnlockFile(ack.token, force)
                    }
                } else {
                    setRefresh('delete')
                }

            }
            catch (err) {
                console.log("rename file error", err);
            }
        }
    }
    const handleUnlockFile = async (token: string, force: boolean) => {
        if (fileAction === 'unlock' && selectedFile) {
            try {
                const unlocked = await unlockFile([token])
                if (unlocked) {
                    handleDeleteFile(force)
                }
            }
            catch (err) {
                console.log("rename file error", err);
            }
        }
    }

    return (
        <Stack spacing={2} sx={{
            display: 'flex',
            flexDirection: 'column',
            height: '100%',
            backgroundColor: 'transparent'
        }}>
            {/* 视图切换和表头 */}
            <Paper elevation={0} sx={{ p: 0, mb: 1, backgroundColor: 'rgb(245,245,245)' }}>
                <Box sx={{
                    display: 'flex',
                    justifyContent: 'flex-start',
                    alignItems: 'center',
                    width: '100%',
                    borderRadius: 1
                }}>
                    {/* 面包屑导航 */}
                    <Stack direction="row"
                        alignItems='center'
                        spacing={0}
                        sx={{
                            borderRadius: 2.5,
                            backgroundColor: 'white',
                            width: '100%',
                            marginRight: 1,
                            padding: 0,
                            overflowX: 'auto',
                            whiteSpace: 'nowrap',
                            scrollbarWidth: 'none',
                        }} >
                        {(filePageProps.category === '' && filePageProps.rootPath === HomePath) && (
                            breadcrumbs.map((crumb, index) => (
                                <>
                                    {index > 0 && (
                                        <NavigateNextIcon fontSize='inherit' />
                                    )}

                                    <IconButton
                                        key={crumb.path}
                                        onClick={() => {
                                            handlePathChange(crumb.path);
                                        }}
                                        sx={{ borderRadius: 2.5 }}
                                    >
                                        {index === 0 ? <HomeIcon sx={{ mr: 0.5 }} fontSize="inherit" /> : null}

                                        <Typography fontSize='small'>{crumb.name}</Typography>
                                    </IconButton>
                                </>
                            ))
                        )}
                        {(filePageProps.category !== '' || filePageProps.rootPath !== HomePath) && (
                            <Stack direction="row"
                                spacing={0}
                                justifyContent='left'
                                alignItems='center'
                                sx={{
                                    borderColor: 'red',
                                }}
                            >
                                <Avatar sx={{ scale: '0.8', maxHeight: '100%', backgroundColor: 'transparent', color: 'black' }}>
                                    {filePageProps.icon}
                                </Avatar>
                                <Typography fontSize='small'>{filePageProps.name}</Typography>
                            </Stack>

                        )}
                    </Stack>

                    <Box sx={{
                        justifyContent: 'center',
                        backgroundColor: 'transparent',
                        borderRadius: 2.5,
                        marginRight: 1,
                    }}>
                        <Stack direction="row" alignItems='center' spacing={1} sx={{
                            borderRadius: 2.5,
                            backgroundColor: 'transparent',
                        }}>
                            <Stack direction="row" spacing={0} sx={{
                                borderRadius: 2.5,
                                backgroundColor: 'white',
                                overflow: 'hidden'
                            }} >
                                <IconButton onClick={handleRefreshFiles} sx={{
                                    width: 45,
                                    borderRadius: 0,
                                    display: 'none'
                                }}>
                                    <RefreshIcon />
                                </IconButton>
                            </Stack>

                            <Stack direction="row" spacing={0} sx={{
                                borderRadius: 2.5,
                                backgroundColor: 'white'
                            }}>
                                <IconButton
                                    onClick={handleViewModeClick}
                                    sx={{
                                        width: 45,
                                        borderRadius: '10px 0 0 10px',
                                    }}
                                >

                                    {viewMode === 'grid' && (
                                        <GridIcon />
                                    )}
                                    {viewMode === 'list' && (
                                        <ListIcon />
                                    )}
                                    {viewMode === 'gallery' && (
                                        <GalleryIcon />
                                    )}
                                </IconButton>
                                {isViewModeOpen && (
                                    <ViewToggle
                                        open={isViewModeOpen}
                                        value={viewMode}
                                        show={showThumb}
                                        pos={viewModePos || { x: 100, y: 20000 }}
                                        onViewModeChange={handleViewModeChange}
                                        onClose={() => setIsViewModeOpen(false)}
                                        onShowThumbChange={handleShowThumbChange}
                                    />
                                )}

                                <Divider orientation="vertical" flexItem />
                                {filePageProps.category === '' && (
                                    <>
                                        <SortMenu sort={orderBy} direction={order} onChange={onSrotChange} />
                                        <Divider orientation="vertical" flexItem />
                                    </>
                                )}
                                <MoreMenu handleRefresh={handleRefreshFiles} />

                            </Stack>
                        </Stack>
                    </Box>



                </Box>
            </Paper >

            {/* 文件列表内容 */}
            < Box sx={{
                flex: 1,
                overflow: 'auto',
                backgroundColor: 'white',
                borderRadius: 2.5,
                padding: 1
            }}>
                {viewMode === 'list' && (
                    <FileListView
                        files={files}
                        onFileClick={onFileClick}
                        onFileRightClick={onFileRightClick}
                    />
                )}

                {
                    viewMode === 'grid' && (
                        <FileGridView
                            files={files}
                            showThumb={showThumb}
                            onFileClick={onFileClick}
                            category={filePageProps.category}
                            onFileRightClick={onFileRightClick}
                        />
                    )
                }

                {
                    viewMode === 'gallery' && (
                        <FileGallaryView
                            files={files}
                            onFileClick={onFileClick}
                        />
                    )
                }
                <ImagePreview
                    open={previewImage.open}
                    imageId={previewImage.url}
                    fetchImage={getURL}
                    onClose={handleClosePreview}
                    sx={{
                        maxWidth: { xs: '100vw', sm: '80vw' },
                        maxHeight: { xs: '100vh', sm: '80vh' }
                    }}
                />
                {(fileMenuPos && selectedFile) && (
                    <FileContextMenu
                        pos={fileMenuPos}
                        handleAction={setFileAction}
                        file={selectedFile}
                        isTrash={filePageProps.rootPath == 'trash'}
                        onClose={() => {
                            setfileMenuPos(null);
                            setFileAction('');
                        }}
                    />
                )}

                {fileAction === 'rename' && (
                    <FileNameDialog
                        open={fileAction === 'rename'}
                        op={FileOp.rename}
                        targetName={selectedFile?.name || ''}
                        onClose={() => { setFileAction('') }}
                        fileType={selectedFile?.type || 0}
                        onSubmit={handleRenameFile}
                    />
                )}


                {fileAction === 'delete' && (
                    <FileDeleteDialog
                        open={fileAction === 'delete'}
                        fileType={selectedFile?.type || 0}
                        fileName={selectedFile?.name || ''}
                        onClose={() => { setFileAction('') }}
                        onDelete={handleDeleteFile}
                    />
                )}


                {fileAction === 'details' && (
                    <FilePreviewDialog
                        open={fileAction === 'details'}
                        uri={selectedFile?.path || ''}
                        thumbUri={selectedFile?.thumbnailUrl || ''}
                        onClose={() => { setFileAction('') }}
                    />
                )}
                {fileAction === 'share' && (
                    <ShareDialog
                        open={fileAction === 'share'}
                        onClose={() => { setFileAction('') }}
                        filePath={selectedFile?.path || ''}
                        shareId=''
                    />
                )}
                {fileAction === "source" && (
                    <LinkDialog
                        open={fileAction === 'source'}
                        onClose={() => { setFileAction('') }}
                        filePath={selectedFile?.path || ''}
                        fileName={selectedFile?.name || ''}
                    />
                )}
                {(fileAction === 'move' || fileAction === 'copy') && (
                    <DirectorySelector
                        open={fileAction === 'move' || fileAction === 'copy'}
                        onClose={() => { setFileAction('') }}
                        op={fileAction}
                        filePath={selectedFile?.path || ''}
                    />
                )}
            </Box >
        </Stack >
    );
};
export default FileListPage;


