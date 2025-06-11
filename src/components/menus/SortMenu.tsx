import { Check } from "@mui/icons-material";
import { IconButton, ListItemText, Menu, MenuItem } from "@mui/material";
import { useState } from "react";
import {
    SwapVert as SortIcon,
} from '@mui/icons-material'

export type Sort = 'name' | 'size' | 'updated_at' | 'created_at';

export type Direction = 'asc' | 'desc';

interface SortItem {
    text: string,
    sort: Sort,
    direction: Direction,
}

const items: SortItem[] = [
    {
        text: 'A-Z',
        sort: 'name',
        direction: 'asc'
    },
    {
        text: 'Z-A',
        sort: 'name',
        direction: 'desc'
    },
    {
        text: '最小',
        sort: 'size',
        direction: 'asc'
    },
    {
        text: '最大',
        sort: 'size',
        direction: 'desc'
    },
    {
        text: '最早修改',
        sort: 'updated_at',
        direction: 'asc'
    },
    {
        text: '最新修改',
        sort: 'updated_at',
        direction: 'desc'
    },
    {
        text: '最早上传',
        sort: 'created_at',
        direction: 'asc'
    },
    {
        text: '最新上传',
        sort: 'created_at',
        direction: 'desc'
    },
]

interface SortMenuProps {
    sort: Sort,
    direction: Direction,
    onChange: (sort: Sort, direction: Direction) => void;
}


const SortMenu: React.FC<SortMenuProps> = ({
    sort, direction, onChange
}) => {
    const [anchorEl, setAnchorEl] = useState(null);
    const isOpen = Boolean(anchorEl);


    const [selectedItem, setSelectedItem] = useState(items.find(item => {
        return item.sort === sort && item.direction === direction;
    }) || items[5])
    // 打开菜单
    const handleClick = (event: any) => {
        setAnchorEl(event.currentTarget);
    };

    const handleItemClick = (item: SortItem) => {
        setSelectedItem(item);
        onChange(item.sort, item.direction);
        handleClose()
    };

    // 关闭菜单
    const handleClose = () => {
        setAnchorEl(null);
    };

    return (
        <>
            <IconButton size='small' onClick={handleClick} sx={{
                borderRadius: 0,
                width: 45,
            }} >
                <SortIcon />
            </IconButton>
            <Menu
                anchorEl={anchorEl}
                open={isOpen}
                onClose={handleClose}

                transformOrigin={{
                    vertical: "top",
                    horizontal: "right",
                }}
                anchorOrigin={{
                    vertical: "bottom",
                    horizontal: "right",
                }}>

                {items.map((item) => (
                    <MenuItem key={item.text} onClick={() => { handleItemClick(item) }}>
                        {item.text == selectedItem.text && (
                            <Check />
                        )}
                        <ListItemText primary={item.text} />
                    </MenuItem>
                ))}
            </Menu >

        </>

    )

}

export default SortMenu;