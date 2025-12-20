import { Box, Paper, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Typography } from "@mui/material";

import { FileItem } from "../../../services/fileService";
import { formatFileSize, getFileIcon } from '../../../utils/file'
import { Share } from "@mui/icons-material";
import TagPreviewMenu from "./TagPreview";

interface FileListViewProps {
    files: FileItem[];
    onFileClick: (file: FileItem) => void;
    onFileRightClick: (event: any, file: FileItem) => void;
}

const FileListView: React.FC<FileListViewProps> = ({
    files,
    onFileClick,
    onFileRightClick,
}) => {
    return <TableContainer component={Paper} sx={{ maxHeight: '100%' }}>
        <Table stickyHeader aria-label="file table">
            <TableHead>
                <TableRow sx={{
                }}>
                    <TableCell size='small'>
                        名称
                    </TableCell>
                    <TableCell size='small' align="right">大小</TableCell>
                    <TableCell size='small' align="right">修改日期</TableCell>
                </TableRow>
            </TableHead>
            <TableBody>
                {files.map((file) => (
                    <TableRow
                        key={file.id}
                        data-id={file.name}
                        hover
                        sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                        onClick={() => { onFileClick(file) }}
                        onContextMenu={(event: any) => { onFileRightClick(event, file) }}
                    >
                        <TableCell size='small' component="th" scope="row">
                            <Box sx={{ display: 'flex', alignItems: 'center' }}>
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
                                <Typography sx={{ ml: 1 }}>{file.name}</Typography>
                                {file.metadata['sys:upload_session_id'] && (
                                    <Typography fontSize={12} sx={{
                                        whiteSpace: 'nowrap',
                                        borderRadius: '8px',
                                        width: '100',
                                        backgroundColor: 'rgb(220,220,220)',
                                        padding: '0px 8px',
                                        marginLeft: 1
                                    }} >上传中</Typography>
                                )}
                                {(!file.metadata['sys:upload_session_id'] && file.tags && file.tags.length > 0) && (
                                    <TagPreviewMenu
                                        tags={file.tags}
                                        maxVisible={1}
                                    />
                                )}
                            </Box>
                        </TableCell>
                        <TableCell size='small' align="right">{file.type == 1 ? '' : formatFileSize(file.size)}</TableCell>
                        <TableCell size='small' align="right">{new Date(file.updated_at).toLocaleString()}</TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    </TableContainer>
}

export default FileListView;