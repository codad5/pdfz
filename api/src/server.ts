import express, { Express, Request, Response, Application } from 'express';
import dotenv from 'dotenv';
import { upload, uploadExists, processedExists, getProcessedFilePath } from '@/helpers/uploadhelper';
import { ResponseHelper } from '@/helpers/response';
import mqConnection, { Queue } from '@/lib/rabbitmq';
import { NewFileProcessQueue, ProcessedFile } from '@/types/queue';
import { ProcessResponse, UploadResponse, ProgressResponse, FinalResponse } from '@/types/response';
import { ProcessOptions } from '@/types/request';
import { getProgress, isFileInProcessing, startFileProcess } from '@/lib/redis';
import fs from 'fs';

dotenv.config();

const app: Application = express();
const port = process.env.API_PORT || 3000;

// Initialize connections before starting server
async function initializeConnections() {
    try {
        await mqConnection.connect();
        console.log('Connected to RabbitMQ');
        return true;
    } catch (error) {
        console.error('Failed to connect to RabbitMQ:', error);
        return false;
    }
}

// Middleware
app.use(express.json());
app.use((req, res, next) => {
    ResponseHelper.registerExpressResponse(req, res);
    next();
});

// Routes
app.get('/', (req: Request, res: Response) => {
    res.send('Welcome to Express & TypeScript Server');
});

app.post('/upload', upload.single('pdf'), async (req: Request, res: Response) => {
    try {
        if (!req.file) {
            throw new Error('File is missing');
        }
        ResponseHelper.success<UploadResponse>({
            id: req.file.filename.split('.').slice(0, -1).join('.'),
            filename: req.file.filename,
            path: req.file.path,
            size: req.file.size
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'File upload failed',
            { message: (error as Error).message ?? 'File upload failed' }
        );
    }
});

app.post('/process/:id', async (req: Request, res: Response) => {
    try {
        const { id } = req.params;
        const params = req.body as ProcessOptions;
        const startPage = params?.startPage ?? 1;
        const pageCount = params?.pageCount ?? 0;
        const priority = params?.priority ?? 1;

        if (!uploadExists(`${id}.pdf`)) {
            throw new Error('File not found');
        }

        if (await isFileInProcessing(id)) {
            console.log('File is already in processing');
            let progress = await getProgress(id);
            if(!progress) {
                progress = 0;
            }
            ResponseHelper.success<ProcessResponse>({
                id,
                file: `${id}.pdf`,
                message: 'File is already in processing',
                options: { startPage, pageCount, priority },
                status: 'processing',
                progress
            });
            return;
        }

        const d = await mqConnection.sendToQueue(Queue.NEW_FILE_EXTRACT, {
            file: `${id}.pdf`,
            start_page: startPage,
            page_count: pageCount, 
            format: 'text',
            engine: 'tesseract',
        });

        if (!d) {
            throw new Error('Failed to send file to queue');
        }

        await startFileProcess(id);

        ResponseHelper.success<ProcessResponse>({
            id,
            file: `${id}.pdf`,
            message: 'File processing started',
            options: { startPage, pageCount, priority },
            status: 'queued',
            progress: 0,
            queuedAt: new Date()
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'File processing failed',
            { message: (error as Error).message ?? 'File processing failed' }
        );
    }
});

app.get('/progress/:id', async (req: Request, res: Response) => {
    try {
        const { id } = req.params;

        if (!uploadExists(`${id}.pdf`)) {
            throw new Error('File not found');
        }

        const progress = await getProgress(id);
        const status = await isFileInProcessing(id) ? 'processing' : 'completed';

        ResponseHelper.success<ProgressResponse>({
            id,
            progress: progress ?? 0,
            status,
            message: 'Progress retrieved successfully'
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'Failed to retrieve progress',
            { message: (error as Error).message ?? 'Failed to retrieve progress' }
        );
    }
});

app.get('/content/:id', async (req: Request, res: Response) => {
    try {
        const { id } = req.params;

        const processedFilePath = getProcessedFilePath(`${id}.json`);

        if (!processedExists(`${id}.json`)) {

            throw new Error('Processed file not found');
        }

        const processedContent = fs.readFileSync(processedFilePath, 'utf-8');
        const content = JSON.parse(processedContent); // Parse the JSON content

        ResponseHelper.success<FinalResponse>({
            id,
            content : content as ProcessedFile,
            message: 'Processed content retrieved successfully',
            status: 'completed'
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'Failed to retrieve processed content',
            { message: (error as Error).message ?? 'Failed to retrieve processed content' }
        );
    }
});

// Start server only after establishing connections
async function startServer() {
    const isConnected = await initializeConnections();
    
    if (isConnected) {
        app.listen(port, () => {
            console.log(`Server is Fire at http://localhost:${port}`);
        });
    } else {
        console.error('Failed to initialize required connections. Exiting...');
        process.exit(1);
    }
}

startServer();