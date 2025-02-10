import express, { Express, Request, Response, Application } from 'express';
import dotenv from 'dotenv';
import bodyParser from 'body-parser';
import { upload, uploadExists, processedExists, getProcessedFilePath } from '@/helpers/uploadhelper';
import { ResponseHelper } from '@/helpers/response';
import mqConnection, { Queue } from '@/lib/rabbitmq';
import { NewFileProcessQueue, OllamaModelPull, ProcessedFile } from '@/types/queue';
import { ProcessResponse, UploadResponse, ProgressResponse, FinalResponse } from '@/types/response';
import { ProcessOptions } from '@/types/request';
import {
    getFileProgress,
    isFileInProcessing,
    startFileProcess,
    startModelDownload,
    isModelDownloading,
    getModelProgress,
    getModelStatus,
    ModelStatus,
    modelDownloadService,
    fileProcessingService
} from '@/lib/redis';
import fs from 'fs';
import { Ollama } from 'ollama';

const ollama = new Ollama({host: process.env.OLLAMA_BASE_URL})

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
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));
app.use(express.json());
app.use((req, res, next) => {
    ResponseHelper.registerExpressResponse(req, res);
    next();
});

// Routes
app.get('/', (req: Request, res: Response) => {
    res.send('PDFz server is life 🔥🔥');
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
        const { startPage = 1, pageCount = 0, priority = 1, engine = 'tesseract' } = req.body as ProcessOptions;
        let model = (req.body as ProcessOptions).model;

        if (!uploadExists(`${id}.pdf`)) {
            throw new Error('File not found');
        }

        if (await fileProcessingService.isFileInProcessing(id)) {
            console.log('File is already in processing');
            const progress = await fileProcessingService.getFileProgress(id) ?? 0;
            
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

        if (engine === 'ollama') {
            if (!model) {
                throw new Error('Model must be specified when using Ollama engine');
            }

            // Add :latest tag if no tag is specified
            if (!model.includes(':')) {
                model = `${model}:latest`;
            }

            const availableModels = await ollama.list();
            const modelExists = availableModels.models.some(m => m.name === model);

            if (!modelExists) {
                throw new Error(
                    `Model ${model} is not available. Please use the /model/pull endpoint to download it.`
                );
            }
        }
        
        const d = await mqConnection.sendToQueue(Queue.NEW_FILE_EXTRACT, {
            file: `${id}.pdf`,
            start_page: startPage,
            page_count: pageCount,
            format: 'text',
            engine,
            model
        });

        if (!d) {
            throw new Error('Failed to send file to queue');
        }

        await fileProcessingService.startFileProcess(id);

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

        const progress = await fileProcessingService.getFileProgress(id);
        const status = await fileProcessingService.isFileInProcessing(id) ? 'processing' : 'completed';

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
        const content = JSON.parse(processedContent);

        ResponseHelper.success<FinalResponse>({
            id,
            content: content as ProcessedFile,
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

app.post('/model/pull', async (req: Request, res: Response) => {
    try {
        const { model } = req.body;

        if (!model) {
            throw new Error('Model name is required');
        }

        // Check if model is already available locally
        const availableModels = await ollama.list();
        console.log("avaliable models", availableModels)
        const modelExists = availableModels.models.some(m => m.name === model);

        if (modelExists) {
            ResponseHelper.success({
                message: 'Model already exists locally',
                model,
                status: 'exists'
            });
            return;
        }

        // Check if model is already in queue or downloading
        const modelStatus = await modelDownloadService.getModelStatus(model);
        if (modelStatus === ModelStatus.QUEUED || modelStatus === ModelStatus.DOWNLOADING) {
            const progress = await getModelProgress(model);
            ResponseHelper.success({
                message: `Model is already ${modelStatus.toLowerCase()}`,
                model,
                status: modelStatus.toLowerCase(),
                progress
            });
            return;
        }

        // Send model pull request to RabbitMQ
        const pullRequest: OllamaModelPull = { name: model };
        const queueResult = await mqConnection.sendToQueue(Queue.OLLAMA_MODEL_PULL, pullRequest);

        if (!queueResult) {
            throw new Error('Failed to queue model download');
        }

        // Initialize model download tracking
        await modelDownloadService.startModelDownload(model);

        ResponseHelper.success({
            message: 'Model download queued successfully',
            model,
            status: 'queued',
            progress: 0
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'Model download failed',
            { message: (error as Error).message ?? 'Model download failed' }
        );
    }
});

app.get('/model/progress/:name', async (req: Request, res: Response) => {
    try {
        const { name } = req.params;

        if (!name) {
            throw new Error('Model name is required');
        }

        const progress = await getModelProgress(name);
        const status = await getModelStatus(name);

        if (!status) {
            throw new Error('Model not found in queue');
        }

        ResponseHelper.success({
            name,
            progress,
            status: status.toLowerCase(),
            message: 'Model progress retrieved successfully'
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'Failed to retrieve model progress',
            { message: (error as Error).message ?? 'Failed to retrieve model progress' }
        );
    }
});

app.get('/models', async (req: Request, res: Response) => {
    try {
        const availableModels = await ollama.list();
        
        ResponseHelper.success({
            message: 'Models retrieved successfully',
            models: availableModels.models.map(m => ({
                name: m.name,
                size: m.size,
                modified_at: m.modified_at
            }))
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'Failed to retrieve models',
            { message: (error as Error).message ?? 'Failed to retrieve models' }
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