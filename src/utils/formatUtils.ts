export const formatDate = (dateString: string): string => {
    const date = new Date(dateString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
};

export const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

export const safeStringToFixed = (str: string, fixed = 1) => {
    const num = +str; // 使用一元运算符转换
    return isNaN(num) ? undefined : num.toFixed(fixed);
}

export const safeStringToCeil = (str: string): number | undefined => {
    const num = +str; // 使用一元运算符转换
    return isNaN(num) ? undefined : Math.ceil(num * 10) / 10;
}

export const calculateMP = (x: string, y: string): string => {
    const width = +x;
    const height = +y;
    if (width <= 0 || height <= 0) {
        throw new Error('分辨率参数必须为正数');
    }

    const mp = (width * height) / 1000000;
    return mp.toFixed(1); // 保留1位小数
}