import { Redis } from 'ioredis';
import { ProcessOptions } from '@/types/request';

enum Status {
    PENDING = "pending",
    DONE = "done",
    FAILED = "failed"
}

const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379');

export async function isFileInProcess(fileId: string): Promise<boolean> {
    const status = await getFileStatus(fileId);
    return status !== Status.PENDING;
}

// check if process is done
export async function isProcessDone(fileId: string): Promise<boolean> {
    const status = await getFileStatus(fileId);
    return status === Status.DONE;
}

async function setStatus(fileId: string, status: Status): Promise<void> {
    await redis.set(`processing:${fileId}`, status.toString());
}

async function getFileStatus(fileId: string): Promise<Status> {
    const status = await redis.get(`processing:${fileId}`);
    // return status enum based on status
    switch (status) {
        case "done":
            return Status.DONE;
        case "failed":
            return Status.FAILED;
        default:
            return Status.PENDING;
    }
}

export async function startFileProcess(
    fileId: string, 
    ttl: number = 3600
): Promise<void> {
    await redis.setex(
        `processing:${fileId}`, 
        ttl,
        Status.PENDING.toString()
    );
}