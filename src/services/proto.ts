export interface AuthReq<T> {
    token: string;
    data: T;
}

export interface CodeError {
    code: number,
    msg: string
}