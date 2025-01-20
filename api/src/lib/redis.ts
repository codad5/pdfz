import { Redis } from 'ioredis';
import { ProcessOptions } from '@/types/request';

const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379');

export async function isFileInProcess(fileId: string): Promise<boolean> {
    const exists = await redis.get(`processing:${fileId}`);
    // if progess is 100, then it's done processing then delete the key and return false
    if (parseInt(exists ?? '') >= 100) {
        return false;
    }
    return !!exists;
}

// check if process is done
export async function isProcessDone(fileId: string): Promise<boolean> {
    const exists = await redis.get(`processing:${fileId}`);
    return parseInt(exists ?? '') >= 100;
}

async function setProgress(fileId: string, progress: number): Promise<void> {
    await redis.set(`processing:${fileId}`, progress);
}

async function getProgress(fileId: string): Promise<number> {
    const progress = await redis.get(`processing:${fileId}`);
    return parseInt(progress ?? '0');
}

export async function markFileAsProcessing(
    fileId: string, 
    ttl: number = 3600
): Promise<void> {
    await redis.setex(
        `processing:${fileId}`, 
        ttl, 
        0 // initial progress 
    );
}