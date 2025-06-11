import React, { useEffect, useState } from 'react';
import {
    Box,
    Typography,
    Paper,
    Divider,
    List,
    ListItem,
    ListItemText,
    useTheme,
    Dialog,
    DialogTitle,
    Stack
} from '@mui/material';
import {
    Close as CloseIcon,
    Camera as ExposureIcon,
    Contrast as ContrastIcon,
    CameraAlt as CameraIcon,

} from '@mui/icons-material';
import { getFileInfo } from '../../services/fileService';
import { FileDetails } from '../../services/fileModel';
import { formatFileSizeWithBytes, getFileIcon, getParentsPath } from '../../utils/file';
import { calculateMP, formatDate, safeStringToCeil, safeStringToFixed } from '../../utils/formatUtils';
import { CloseButton } from './dialog';


interface FilePreviewProps {
    open: boolean,
    uri: string;
    thumbUri: string,
    onClose: () => void;
}

const FilePreviewDialog: React.FC<FilePreviewProps> = ({ open, uri, thumbUri: thumbUrl, onClose }) => {
    const theme = useTheme();
    const [file, setFile] = useState<FileDetails>({} as FileDetails);

    useEffect(() => {
        if (uri) {
            const getFileDetails = async () => {
                const file = await getFileInfo(uri);
                setFile(file);
            }
            getFileDetails();
        }
    }, [uri])

    return (
        <Dialog
            open={open}
            onClose={onClose}
            maxWidth="sm"
            fullWidth
            aria-labelledby="file-detail-dialog-title"
        >
            <CloseButton onClick={onClose} aria-label="关闭预览">
                <CloseIcon fontSize="large" />
            </CloseButton>

            <Box sx={{
                display: 'flex',
                flexDirection: 'column',
                height: '100vh',
                backgroundColor: theme.palette.background.default
            }}>
                <DialogTitle id="file-detail-dialog-title" sx={{ p: 2, pb: 1 }}>
                    <Box display="flex" alignItems="center" justifyContent="space-between">
                        <Box sx={{
                            display: 'flex',
                            justifyContent: 'center',
                            alignItems: 'center'
                        }}>

                            {getFileIcon(file.type, file.name)}

                            <Typography variant="h6" component="div">
                                {file.name}
                            </Typography>
                        </Box>


                    </Box>
                </DialogTitle>

                {/* 预览区域 */}
                {(file.type === 0 && thumbUrl) && (
                    <img
                        src={thumbUrl}
                        alt={file.name}
                        style={{
                            maxWidth: '100%',
                            maxHeight: '100%',
                            objectFit: 'fill'
                        }}
                    />
                )}
                {/* 媒体信息 */}
                {(file.metadata && file.metadata["exif:x"]) && (
                    <Paper square elevation={0} sx={{
                        p: 3,
                        borderTop: `1px solid ${theme.palette.divider}`,
                        backgroundColor: 'white'
                    }}>

                        <Typography variant="subtitle1" gutterBottom>
                            媒体信息
                        </Typography>
                        <Divider sx={{ mb: 2 }} />
                        <List dense>
                            <ListItem disablePadding sx={{
                                borderRadius: 4,
                                backgroundColor: 'rgb(245, 245, 245)',
                                py: 1, marginBottom: 1,
                            }}>
                                <Stack direction='row' spacing={2} sx={{
                                    justifyContent: 'center',
                                    alignItems: 'center',
                                    display: 'flex',
                                    width: '100%',
                                    marginLeft: 2,
                                    marginRight: 2
                                }}>
                                    <ExposureIcon htmlColor='gray' />
                                    <Box sx={{
                                        justifyContent: 'space-between',
                                        alignItems: 'center',
                                        display: 'flex',
                                        width: '100%',
                                    }}>
                                        <ListItemText sx={{
                                            display: 'inline-block',
                                            width: 'unset',
                                            flex: '0 1 auto',
                                        }} primary="光圈" secondary={safeStringToCeil(file.metadata['exif:f'])}></ListItemText>
                                        <ListItemText primary="曝光" sx={{
                                            display: 'inline-block',
                                            width: 'unset',
                                            flex: '0 1 auto',
                                        }}
                                            secondary={file.metadata['exif:exposure_time']}></ListItemText>
                                        <ListItemText primary="ISO" sx={{
                                            display: 'inline-block',
                                            width: 'unset',
                                            flex: '0 1 auto',
                                        }}
                                            secondary={file.metadata['exif:iso']}></ListItemText>
                                    </Box>
                                </Stack>
                            </ListItem>

                            <ListItem disablePadding sx={{
                                borderRadius: 4,
                                backgroundColor: 'rgb(245, 245, 245)',
                                py: 1, marginBottom: 1
                            }}>
                                <Stack direction='row' spacing={2} sx={{
                                    justifyContent: 'center',
                                    alignItems: 'center',
                                    display: 'flex',
                                    width: '100%',
                                    marginLeft: 2,
                                    marginRight: 2
                                }}>
                                    <ContrastIcon htmlColor='gray' />
                                    <Box sx={{
                                        justifyContent: 'space-between',
                                        alignItems: 'center',
                                        display: 'flex',
                                        width: '100%',
                                    }}>
                                        <ListItemText sx={{
                                            display: 'inline-block',
                                            width: 'unset',
                                            flex: '0 1 auto',
                                        }} primary="曝光补偿"
                                            secondary={safeStringToFixed(file.metadata['exif:exposure_bias']) + ' ev'} />
                                        <ListItemText sx={{
                                            flex: '0 1 auto',
                                            display: 'inline-block',
                                            width: 'unset'

                                        }}
                                            primary="闪光灯"
                                            secondary={file.metadata['exif:flash'] === '0' ? '关闭' : '打开'} />
                                    </Box>
                                </Stack>
                            </ListItem>
                            <ListItem disablePadding sx={{
                                borderRadius: 4,
                                backgroundColor: 'rgb(245, 245, 245)',
                                py: 1, marginBottom: 1
                            }}>
                                <Stack direction='row' spacing={2} sx={{
                                    justifyContent: 'center',
                                    alignItems: 'center',
                                    display: 'flex',
                                    width: '100%',
                                    marginLeft: 2,
                                    marginRight: 2
                                }}>
                                    <CameraIcon htmlColor='gray' />
                                    <ListItemText
                                        primary={`${file.metadata['exif:camera_make']} ${file.metadata['exif:camera_model']}`}
                                        secondary={file.metadata['exif:focal_length'] + ' mm'}
                                    />
                                </Stack>
                            </ListItem>
                            <ListItem disablePadding sx={{
                                borderRadius: 4,
                                backgroundColor: 'rgb(245, 245, 245)',
                                py: 1, marginBottom: 1
                            }}>
                                <Stack direction='row' spacing={2} sx={{
                                    justifyContent: 'center',
                                    alignItems: 'center',
                                    display: 'flex',
                                    width: '100%',
                                    marginLeft: 2,
                                    marginRight: 2
                                }}>
                                    <CameraIcon htmlColor='gray' />
                                    <ListItemText
                                        primary='拍摄时间'
                                        secondary={formatDate(file.metadata['exif:taken_at'])}
                                    />
                                </Stack>
                            </ListItem>

                            <ListItem disablePadding sx={{
                                borderRadius: 4,
                                backgroundColor: 'rgb(245, 245, 245)',
                                py: 1, marginBottom: 1
                            }}>
                                <Stack direction='row' spacing={2} sx={{
                                    justifyContent: 'center',
                                    alignItems: 'center',
                                    display: 'flex',
                                    width: '100%',
                                    marginLeft: 2,
                                    marginRight: 2
                                }}>
                                    <CameraIcon htmlColor='gray' />
                                    <ListItemText
                                        primary='分辨率'
                                        secondary={` ${calculateMP(file.metadata['exif:x'], file.metadata['exif:y'])}MP · ${file.metadata['exif:x']} x ${file.metadata['exif:y']}`}
                                    />
                                </Stack>
                            </ListItem>

                            <ListItem disablePadding sx={{
                                borderRadius: 4,
                                backgroundColor: 'rgb(245, 245, 245)',
                                py: 1, marginBottom: 1
                            }}>
                                <Stack direction='row' spacing={2} sx={{
                                    justifyContent: 'center',
                                    alignItems: 'center',
                                    display: 'flex',
                                    width: '100%',
                                    marginLeft: 2,
                                    marginRight: 2
                                }}>
                                    <CameraIcon htmlColor='gray' />
                                    <ListItemText
                                        primary='软件'
                                        secondary={file.metadata['exif:software']}
                                    />
                                </Stack>
                            </ListItem>

                        </List>

                    </Paper>
                )}
                {/* 文件信息 */}
                <Paper square elevation={0} sx={{
                    p: 3,
                    borderTop: `1px solid ${theme.palette.divider}`
                }}>
                    <Typography variant="subtitle1" gutterBottom>
                        基本信息
                    </Typography>
                    <Divider sx={{ mb: 2 }} />

                    <List dense>
                        <ListItem disablePadding>
                            <ListItemText
                                primary="类型"
                                secondary={file.type === 1 ? '文件夹' : `文件`}
                            />
                        </ListItem>
                        <Divider component="li" />

                        <ListItem disablePadding>
                            <ListItemText primary="所在目录" secondary={getParentsPath(file.path)} />
                        </ListItem>
                        <Divider component="li" />
                        {file.type === 0 && (
                            <ListItem disablePadding>
                                <ListItemText primary="大小" secondary={formatFileSizeWithBytes(file.size)} />
                            </ListItem>
                        )}
                        {(file.type === 0 && file.extended_info?.storage_used) && (
                            <>
                                <Divider component="li" />

                                <ListItem disablePadding>
                                    <ListItemText primary="占用空间" secondary={formatFileSizeWithBytes(file.extended_info?.storage_used)} />
                                </ListItem>
                            </>
                        )}

                        <Divider component="li" />

                        <ListItem disablePadding>
                            <ListItemText primary="创建时间" secondary={formatDate(file.created_at)} />
                        </ListItem>
                        <Divider component="li" />

                        <ListItem disablePadding>
                            <ListItemText primary="修改时间" secondary={formatDate(file.updated_at)} />
                        </ListItem>
                    </List>
                </Paper>
            </Box>
        </Dialog >

    );
};

export default FilePreviewDialog;