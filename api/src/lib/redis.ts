import { Redis } from 'ioredis';
import { ProcessOptions } from '@/types/request';

enum Status {
    PENDING = "pending",
    DONE = "done",
    FAILED = "failed"
}

const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379');

export async function isFileInProcessing(fileId: string): Promise<boolean> {
    const status = await getFileStatus(fileId);
    console.log(status, "meant to be pe");
    return status === Status.PENDING;
}

// check if process is done
export async function isProcessDone(fileId: string): Promise<boolean> {
    const status = await getFileStatus(fileId);
    return status === Status.DONE;
}

async function setStatus(fileId: string, status: Status): Promise<void> {
    await redis.set(`processing:${fileId}`, status.toString());
}

async function getFileStatus(fileId: string): Promise<Status|null> {
    const status = await redis.get(`processing:${fileId}`);
    console.log("status", status);
    // return status enum based on status
    switch (status) {
        case "done":
            return Status.DONE;
        case "failed":
            return Status.FAILED;
        case 'pending':
            return Status.PENDING;
        default:
            return null;
        
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

// Mark a file as done
export async function markAsDone(fileId: string): Promise<void> {
    await setStatus(fileId, Status.DONE);
}

// Mark a file as failed
export async function markAsFailed(fileId: string): Promise<void> {
    await setStatus(fileId, Status.FAILED);
}

// Mark progress for a file
export async function markProgress(fileId: string, page: number, total: number): Promise<void> {
    const progress = Math.floor((page / total) * 100);
    await redis.set(`progress:${fileId}`, progress);

    if (progress === 100) {
        await markAsDone(fileId);
    }
}

// Get progress for a file
export async function getProgress(fileId: string): Promise<number> {
    const progress = await redis.get(`progress:${fileId}`);
    return progress ? parseInt(progress, 10) : 0;
}

// Mark a file with a specific status
export async function markAs(fileId: string, status: Status): Promise<void> {
    await setStatus(fileId, status);
}
