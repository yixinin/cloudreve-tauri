import { Box, Card, CardMedia, Grid, IconButton, Typography } from "@mui/material";

import { FileItem } from "../../../services/fileService";
import { getFileIcon, renderThumbnail } from "../../../utils/file";
import { Share } from "@mui/icons-material";
import TagPreviewMenu from "./TagPreview";


interface FileGridViewProps {
    files: FileItem[];
    category: string,
    showThumb: boolean,
    onFileClick: (file: FileItem) => void;
    onFileRightClick: (event: any, file: FileItem) => void;
}

const FileGridView: React.FC<FileGridViewProps> = ({
    files,
    category,
    showThumb,
    onFileClick,
    onFileRightClick,
}) => {
    return <Box sx={{
        height: '100vh',
        display: 'flex',
        flexDirection: 'column',
    }}>
        <Typography sx={{ margin: '10px', display: files.filter(file => file.type === 1).length > 0 ? 'block' : 'none' }}>文件夹</Typography>
        <Grid container spacing={1}>
            {
                files.filter(file => file.type === 1).map((file) => (
                    <Grid key={file.id} size={{ xs: 6, sm: 4, md: 3, lg: 2 }}
                        onClick={() => {
                            onFileClick(file)
                        }}
                        onContextMenu={(event: any) => { onFileRightClick(event, file) }}
                    >
                        <IconButton size='large' sx={{
                            display: 'flex', alignItems: 'center', justifyContent: 'flex-start',
                            borderRadius: '10px',
                            backgroundColor: 'action.hover',
                            width: '100%'
                        }} >
                            {getFileIcon(file.type, file.name)}
                            <Typography sx={{ marginLeft: '10px' }} fontSize={15} variant="body2" noWrap>
                                {file.name}
                            </Typography>
                        </IconButton>
                    </Grid>
                ))}
        </Grid>
        {category === '' && (
            < Typography sx={{
                margin: '10px',
                display: files.filter(file => file.type === 0).length > 0 ? 'block' : 'none'
            }}
            >文件</Typography>
        )}

        <Grid container spacing={1} sx={{ pb: 6 }}>
            {files.filter((file) => file.type === 0).map((file) => (
                <Grid key={file.id} size={{ xs: 6, sm: 4, md: 3, lg: 2 }}
                    onClick={() => {
                        onFileClick(file)
                    }}
                    onContextMenu={(event: any) => { onFileRightClick(event, file) }}
                >
                    <Card sx={{
                        display: 'flex',
                        flexDirection: 'column',
                        backgroundColor: 'action.hover',
                    }}>

                        <Box sx={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: "center",
                        }}>
                            <Box sx={{
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'flex-start',
                                padding: 1,
                                overflow: 'hidden'
                            }}>
                                <Box sx={{
                                    display: 'flex',
                                }}>
                                    {getFileIcon(file.type, file.name)}
                                    {file.shared && (
                                        <Share htmlColor="gray" sx={{
                                            // position: 'absolute',
                                            backgroundColor: 'white',
                                            scale: '0.5',
                                            borderRadius: 3,
                                            marginLeft: -2,
                                            marginTop: 0.5,
                                        }} />
                                    )}
                                </Box>

                                <Typography fontSize={12} maxWidth='80%' variant="body2" noWrap>
                                    {file.name}
                                </Typography>

                            </Box>
                            {file.metadata['sys:upload_session_id'] && (
                                <Typography fontSize={12} sx={{
                                    whiteSpace: 'nowrap',
                                    borderRadius: '8px',
                                    width: '100',
                                    backgroundColor: 'rgb(220,220,220)',
                                    padding: '0px 8px',
                                    marginRight: 1
                                }} >上传中</Typography>
                            )}
                            {(!file.metadata['sys:upload_session_id'] && file.tags && file.tags.length > 0) && (
                                <TagPreviewMenu
                                    tags={file.tags}
                                    maxVisible={1}
                                />
                            )}
                        </Box>


                        {showThumb && (
                            <CardMedia
                                component="div"
                                sx={{
                                    pt: '100%',
                                    position: 'relative',
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    alignContent: 'center'
                                }}
                            >
                                {renderThumbnail(file)}
                            </CardMedia>
                        )}

                    </Card>
                </Grid>
            ))}
        </Grid>
    </Box>
}

export default FileGridView;