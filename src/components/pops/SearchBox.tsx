import { Search } from "@mui/icons-material";
import { Box, Dialog, IconButton, TextField } from "@mui/material";
import { useState } from "react";

interface SearchBoxProps {
    open: boolean;
    onClose: () => void;
    onSearch: (name: string) => void;
}
export const SearchBox: React.FC<SearchBoxProps> = ({
    open, onClose, onSearch
}) => {

    const [name, setName] = useState('');
    const handleClose = () => {
        setName('');
        onClose();
    };

    const handleSearchClick = () => {
        onSearch(name)
        handleClose()
    }

    const onTextChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        setName(event.target.value)
    }

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            handleSearchClick();
        }
    };

    return <Dialog
        open={open}
        onClose={handleClose}
        maxWidth="sm"
        fullWidth
        aria-labelledby="search-box-dialog-title"
        sx={{
            // display: 'flex',
            flexDirection: 'column',
            alignItems: 'start', // 内容左对齐
            maxHeight: '40vh',       // 限制高度避免溢出
            overflow: 'auto',        // 允许滚动
            paddingTop: 0,           // 移除默认顶部内边距
        }}
    >
        <Box sx={{
            display: 'flex',
            padding: 2,
            position: 'relative',
        }}>
            <TextField
                fullWidth
                value={name}
                onChange={onTextChange}
                onKeyDown={handleKeyDown}
                label="搜索文件名"
                sx={{

                }}
            >

            </TextField>

            <IconButton onClick={handleSearchClick} sx={{
                borderRadius: 0
            }}>
                <Search />
            </IconButton>
        </Box>



    </Dialog>
};

export default SearchBox;