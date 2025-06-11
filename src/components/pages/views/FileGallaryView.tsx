import { Card, CardMedia, Grid } from "@mui/material";

import { FileItem } from "../../../services/fileService";
import { renderThumbnail } from '../../../utils/file';

interface FileGallaryProps {
    files: FileItem[];
    onFileClick: (file: FileItem) => void;
}

const FileGallaryView: React.FC<FileGallaryProps> = ({
    files,
    onFileClick
}) => {
    return <Grid container spacing={0}>
        {files.map((file) => (
            <Grid key={file.id} size={{ xs: 6, sm: 4, md: 3, lg: 2 }}
                onClick={() => {
                    onFileClick(file)
                }}>
                <Card sx={{
                    display: 'flex',
                    flexDirection: 'column',
                    backgroundColor: 'action.hover',
                    borderRadius: 0
                }}>
                    <CardMedia
                        component="div"
                        sx={{
                            pt: '100%',
                            position: 'relative',
                            // display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            borderRadius: 0
                        }}
                    >
                        {renderThumbnail(file)}
                    </CardMedia>
                </Card>
            </Grid>
        ))}
    </Grid>
}

export default FileGallaryView;