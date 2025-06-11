import React, { useState, MouseEvent } from 'react';
import Menu from '@mui/material/Menu';

interface Position {
    mouseX: number;
    mouseY: number;
}

const EmptyContextMenu: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [contextMenu, setContextMenu] = useState<Position | null>(null);
    console.log("children: ", children);

    const handleContextMenu = (event: MouseEvent) => {
        event.preventDefault();
        setContextMenu(null);
    };

    const handleClose = () => {
        setContextMenu(null);
    };



    return (
        <div onContextMenu={handleContextMenu} style={{ cursor: 'context-menu' }}>
            {children}
            <Menu
                open={contextMenu !== null}
                onClose={handleClose}
                anchorReference="anchorPosition"
                anchorPosition={
                    contextMenu !== null
                        ? { top: contextMenu.mouseY, left: contextMenu.mouseX }
                        : undefined
                }
            >

            </Menu>
        </div>
    );
};

export default EmptyContextMenu;