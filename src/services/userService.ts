import { invoke } from '@tauri-apps/api/core';

export interface LoginAck {
    user: User;
    token: Token;
}

export interface User {
    id: string;
    email: string;
    nickname: string;
    status: string;
    created_at: string;
    group: Group;
    language: string;
}

export interface Group {
    id: string;
    name: string;
    permission: string;
    direct_link_batch_size: number;
    trash_retention: number;
}

export interface Token {
    access_token: string;
    refresh_token: string;
    access_expires: string;
    refresh_expires: string;
}

export interface PrepareAck {
    webauthn_enabled: boolean;
    password_enabled: boolean;
}

export const userLogin = async (username: string, password: string): Promise<LoginAck> => {
    try {
        const ack = await invoke<LoginAck>('login', { username, password });
        return ack;
    } catch (error) {
        console.error('login error:', error);
        throw error;
    }
};

export const prepare = async (addr: string, username: string): Promise<PrepareAck> => {
    try {
        const ack = await invoke<PrepareAck>('prepare', { addr, username });
        return ack;
    } catch (error) {
        console.error('prepare error:', error);
        throw error;
    }
};

export const userLogout = async (): Promise<boolean> => {
    try {
        const ack = await invoke<boolean>('logout', {});
        return ack;
    } catch (error) {
        console.error('logout error:', error);
        throw error;
    }
}

export interface LoginData {
    username: string,
    password: string,
}

export interface NetworkSettings {
    addr: string,
    addr6: string,
    mode: NetworkMode,
}

export enum NetworkMode {
    Auto = 1,
    Normal = 2,
    IPv6 = 3,
    P2P = 4,
}

export const getNetworkSetting = async (): Promise<NetworkSettings> => {
    try {
        const ack = await invoke<NetworkSettings>('get_network_settings', {});
        return ack;
    } catch (error) {
        console.error('get network setting error:', error);
        throw error;
    }
}

export const setNetworkSettings = async (addr: string, addr6: string, mode: NetworkMode): Promise<boolean> => {
    try {
        const ack = await invoke<boolean>('set_network_settings', { addr, addr6, mode });
        return ack;
    } catch (error) {
        console.error('save network setting error:', error);
        throw error;
    }
}


export const setNetworkSettingsMode = async (mode: NetworkMode): Promise<boolean> => {
    try {
        const ack = await invoke<boolean>('set_network_settings', { mode });
        return ack;
    } catch (error) {
        console.error('save network mode error:', error);
        throw error;
    }
}

export const getLoginData = async (): Promise<LoginData> => {
    try {
        const ack = await invoke<LoginData>('get_login_data', {});
        return ack;
    } catch (error) {
        console.error('get login data error:', error);
        throw error;
    }
}