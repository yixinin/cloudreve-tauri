import { FileItem } from "../services/fileService";

import imageIcon from "../assets/image.svg"
import audioIcon from "../assets/audio.svg"
import videoIcon from "../assets/video.svg"
import docIcon from "../assets/doc.svg"
import fileIcon from "../assets/file.svg"


import {
    Folder as FolderIcon,
    InsertDriveFile as FileIcon,
    Image as ImageIcon,
    Description as DocIcon,
    MusicNote as AudioIcon,
    VideocamRounded as VideoIcon,

} from '@mui/icons-material';
import { Box } from "@mui/material";
import { FileTag } from "../services/fileModel";

export enum FileKind {
    folder,
    image,
    audio,
    video,
    document,
    archive,
    ppt,
    excel,
    unknown,
}

export const fileCanOpen = (file: FileItem) => {
    switch (getFileKind(file.type, file.name)) {
        case FileKind.audio:
        case FileKind.video:
        case FileKind.image:
        case FileKind.folder:
            return true
    }
    return false
}

// 渲染缩略图或占位符
export const renderThumbnail = (file: FileItem) => {
    if (file.type === 1) {
        return (
            <Box sx={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                padding: 0,
                margin: 0,
            }}>
                <FolderIcon htmlColor="gray" fontSize="large" sx={{
                    scale: 2
                }} />
            </Box>
        );
    }

    if (file.thumbnailUrl) {
        var scale = '30%'
        var offset = '35%'
        if (file.thumbnailUrl.startsWith("http")) {
            scale = '100%'
            offset = '0'
        }
        return (
            <Box
                component="img"
                src={file.thumbnailUrl}
                alt={file.name}
                sx={{
                    width: scale,
                    height: scale,
                    objectFit: 'cover',
                    borderRadius: '0',
                    position: 'absolute',
                    top: offset,
                    left: offset,
                    alignSelf: 'center',
                    padding: 0,
                    margin: 0,
                }
                }
                onError={(e) => {
                    // 图片加载失败时显示默认图标
                    const target = e.target as HTMLImageElement;
                    target.style.display = 'none';
                }}
            />
        );
    }
}


export const getFileIcon = (fileType: number, name: string) => {
    if (fileType === 1) return <FolderIcon sx={{ color: 'gray' }} />;
    switch (getFileKind(fileType, name)) {
        case FileKind.image: return <ImageIcon sx={{ color: 'rgb(211, 47, 47)' }} />;
        case FileKind.document: return <DocIcon sx={{ color: 'rgb(96, 125, 139)' }} />;
        case FileKind.audio: return <AudioIcon sx={{ color: 'rgb(101, 31, 255)' }} />;
        case FileKind.video: return <VideoIcon sx={{ color: 'rgb(213, 0, 0)' }} />;
        default: return <FileIcon sx={{ color: 'gray' }} />;
    }
}

export const getFileKind = (fileType: number, fileName: string) => {
    if (!fileName) {
        return FileKind.unknown
    }
    if (fileType == 1) {
        return FileKind.folder
    }
    const extension = fileName.split('.').pop()?.toLowerCase();
    switch (extension) {
        case 'png':
        case 'jpg':
        case 'jpeg':
        case 'gif':
        case "webp":
        case "bmp":
            return FileKind.image
        case 'mp3':
        case 'wav':
        case "flac":
        case "aac":
        case "ogg":
            return FileKind.audio
        case 'doc':
        case 'txt':
        case 'pdf':
        case 'md':
            return FileKind.document
        case 'mp4':
        case 'avi':
        case 'mkv':
        case 'h264':
        case 'h265':
        case "webm":
        case "mov":
            return FileKind.video
    }
    return FileKind.unknown
}

export const getDefaultThumb = (fileType: number, fileName: string) => {
    const kind = getFileKind(fileType, fileName)
    if (kind == FileKind.image) {
        return imageIcon
    }
    if (kind == FileKind.video) {
        return videoIcon
    }
    if (kind == FileKind.audio) {
        return audioIcon
    }
    if (kind == FileKind.document) {
        return docIcon
    }
    return fileIcon
}


export const formatFileSize = (bytes: number, decimals: number = 2): string => {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    // 处理小数位数
    const dm = decimals < 0 ? 0 : decimals;
    const formattedSize = parseFloat((bytes / Math.pow(k, i)).toFixed(dm));

    return `${formattedSize} ${sizes[i]}`;
}

export const formatFileSizeWithBytes = (bytes: number, decimals: number = 2): string => {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    // 处理小数位数
    const dm = decimals < 0 ? 0 : decimals;
    const formattedSize = parseFloat((bytes / Math.pow(k, i)).toFixed(dm));

    return `${formattedSize} ${sizes[i]} (${bytes} 字节)`;
}
export const getParentsPath = (path: string) => {
    if (path && path !== '') {
        try {
            const url = new URL(path);
            const end = url.pathname.lastIndexOf('/')
            const parents = url.pathname.substring(0, end);

            switch (url.host) {
                case "my":
                    return `我的文件${parents}`
            }
            return parents
        } catch {
            return path
        }
    }
    return ''
}
export const getFileTags = (file: FileItem) => {
    var tags: FileTag[] = [];
    Object.entries(file.metadata).forEach(([key, value]) => {
        if (key.startsWith("tag:")) {
            tags.push({
                key: key.substring(4),
                color: value,
            })
        }
    });
    return tags
}