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

export const userLogin = async (addr: string, ipv6: boolean, addr6: string, username: string, password: string): Promise<LoginAck> => {
    try {
        const ack = await invoke<LoginAck>('login', { addr, ipv6, addr6, username, password });
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

export interface UserSessting {
    force_ipv4: boolean,
    force_ipv6: boolean,
}


export interface LoginData {
    addr: string,
    ipv6: boolean,
    addr6: string,
    username: string,
    password: string,
}

export const getUserSetting = async (): Promise<UserSessting> => {
    try {
        const ack = await invoke<UserSessting>('get_user_setting', {});
        return ack;
    } catch (error) {
        console.error('get user setting error:', error);
        throw error;
    }
}

export const saveUserSetting = async (forceIpv6: boolean, forceIpv4: boolean): Promise<boolean> => {
    try {
        const ack = await invoke<boolean>('save_user_setting', { forceIpv6, forceIpv4 });
        return ack;
    } catch (error) {
        console.error('save user setting error:', error);
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