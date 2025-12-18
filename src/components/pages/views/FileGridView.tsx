import { Box, Card, CardMedia, IconButton, Typography } from "@mui/material";
import { Grid } from "react-window";

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
    // 获取文件夹和文件
    const folders = files.filter(file => file.type === 1);
    const regularFiles = files.filter(file => file.type === 0);

    // 计算列宽
    const getColumnWidth = () => {
        const width = window.innerWidth;
        if (width < 600) return width / 2 - 10; // xs: 6
        if (width < 960) return width / 4 - 10; // sm: 4
        if (width < 1280) return width / 3 - 10; // md: 3
        return width / 2 - 10; // lg: 2
    };

    // 计算行高
    const getRowHeight = (index: number) => {
        // 文件夹项高度
        if (index < folders.length) return 60;
        // 文件项高度（带缩略图或不带）
        return showThumb ? 150 : 60;
    };

    // 渲染网格项
    const GridItem = ({ columnIndex, rowIndex, style }: any) => {
        const itemIndex = rowIndex * 6 + columnIndex; // 假设最大6列
        let item: FileItem | undefined;

        // 确定是文件夹还是文件
        if (itemIndex < folders.length) {
            item = folders[itemIndex];
        } else if (itemIndex < folders.length + regularFiles.length) {
            item = regularFiles[itemIndex - folders.length];
        }

        if (!item) return <div style={style} />;

        return (
            <div style={style}>
                {item.type === 1 ? (
                    // 文件夹项
                    <IconButton size='large' sx={{
                        display: 'flex', alignItems: 'center', justifyContent: 'flex-start',
                        borderRadius: '10px',
                        backgroundColor: 'action.hover',
                        width: '100%'
                    }}
                        onClick={() => onFileClick(item)}
                        onContextMenu={(event: any) => onFileRightClick(event, item)}
                    >
                        {getFileIcon(item.type, item.name)}
                        <Typography sx={{ marginLeft: '10px' }} fontSize={15} variant="body2" noWrap>
                            {item.name}
                        </Typography>
                    </IconButton>
                ) : (
                    // 文件项
                    <Card sx={{
                        display: 'flex',
                        flexDirection: 'column',
                        backgroundColor: 'action.hover',
                        height: '100%'
                    }}
                        onClick={() => onFileClick(item)}
                        onContextMenu={(event: any) => onFileRightClick(event, item)}
                    >
                        <Box sx={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: "center",
                            padding: 1
                        }}>
                            <Box sx={{
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'flex-start',
                                overflow: 'hidden'
                            }}>
                                <Box sx={{
                                    display: 'flex',
                                }}>
                                    {getFileIcon(item.type, item.name)}
                                    {item.shared && (
                                        <Share htmlColor="gray" sx={{
                                            backgroundColor: 'white',
                                            scale: '0.5',
                                            borderRadius: 3,
                                            marginLeft: -2,
                                            marginTop: 0.5,
                                        }} />
                                    )}
                                </Box>

                                <Typography fontSize={12} maxWidth='80%' variant="body2" noWrap>
                                    {item.name}
                                </Typography>
                            </Box>
                            {item.metadata['sys:upload_session_id'] && (
                                <Typography fontSize={12} sx={{
                                    whiteSpace: 'nowrap',
                                    borderRadius: '8px',
                                    width: '100',
                                    backgroundColor: 'rgb(220,220,220)',
                                    padding: '0px 8px',
                                    marginRight: 1
                                }} >上传中</Typography>
                            )}
                            {(!item.metadata['sys:upload_session_id'] && item.tags && item.tags.length > 0) && (
                                <TagPreviewMenu
                                    tags={item.tags}
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
                                    alignContent: 'center',
                                    flex: 1
                                }}
                            >
                                {renderThumbnail(item)}
                            </CardMedia>
                        )}
                    </Card>
                )}
            </div>
        );
    };

    // 计算总项目数和列数
    const totalItems = folders.length + regularFiles.length;
    const maxColumns = 6;
    const totalRows = Math.ceil(totalItems / maxColumns);

    return (
        <Box sx={{
            height: 'calc(100vh - 200px)',
            display: 'flex',
            flexDirection: 'column',
        }}>
            <Typography sx={{ margin: '10px', display: folders.length > 0 ? 'block' : 'none' }}>文件夹</Typography>

            {category === '' && (
                <Typography sx={{
                    margin: '10px',
                    display: regularFiles.length > 0 ? 'block' : 'none'
                }}>
                    文件
                </Typography>
            )}

            {totalItems > 0 ? (
                <Grid
                    columnCount={maxColumns}
                    columnWidth={getColumnWidth}
                    rowCount={totalRows}
                    rowHeight={getRowHeight}
                    style={{ height: '100%', width: '100%', marginTop: 10 }}
                    cellComponent={GridItem}
                    cellProps={{} as any}
                />
            ) : (
                <Typography sx={{ textAlign: 'center', marginTop: 2 }}>无文件</Typography>
            )}
        </Box>
    )
}

export default FileGridView;