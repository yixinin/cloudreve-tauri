
import {
    IconButton,
    styled,
} from "@mui/material";



export const CloseButton = styled(IconButton)(({ theme }) => ({
    position: 'absolute',
    top: theme.spacing(2),
    right: theme.spacing(2),
    color: 'white',
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    '&:hover': {
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
    },
}));


export default CloseButton;