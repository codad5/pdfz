# PDFz

Developed by [codad5](https://github.com/codad5)

PDFz streamlines the extraction and processing of text from PDF files so that you can manage and analyze large volumes of documents effortlessly. By leveraging a microservices architecture, PDFz achieves high performance through:

- **Extractor Service (Rust):** Processes PDF files and extracts text using configurable extraction engines. While Tesseract OCR is supported, PDFz is designed to work with multiple extraction methods.
- **API Service (Express & TypeScript):** Provides endpoints for file uploads, processing, progress tracking, and interacting with advanced extraction and model-based processing.
- **Redis:** Caches and tracks file and model processing progress.
- **RabbitMQ:** Manages message queuing between services.
- **Model-Based Processing:** Integrate with engines like Ollama for advanced text processing using locally hosted large language models (LLMs).

---

## Features

- **File Upload:** Send PDF files to the API.
- **Multi-Engine File Processing:** Choose your extraction engine—whether Tesseract OCR, Ollama, or others—to process PDFs asynchronously.
- **OCR & Model-Based Extraction:**  
  - Use Tesseract OCR for traditional optical character recognition.
  - Leverage model-based extraction (e.g., using Ollama) for advanced processing such as summarization, question-answering, or generating insights.
- **Progress Tracking:** Monitor file processing progress in real time.
- **Processed Content Retrieval:** Get back JSON with extracted content.
- **Model Management:**  
  - Pull and download a specified model if it isn’t available locally.
  - Track model download progress.
  - List available models for advanced extraction needs.

---

## Architecture

- **API Service (Express & TypeScript):**  
  Provides endpoints for:
  - Web Interface files (`/web`)
  - Uploading files (`/upload`)
  - Initiating file processing (`/process/:id`)
  - Checking file processing progress (`/progress/:id`)
  - Retrieving processed content (`/content/:id`)
  - Managing models (pulling via `/model/pull`, tracking progress with `/model/progress/:name`, and listing models with `/models`)

- **Extractor Service (Rust):**  
  Processes queued PDF files using the chosen extraction engine. It supports both traditional OCR (e.g., Tesseract) and model-based extraction (e.g., via Ollama) and interacts with Redis and RabbitMQ for job tracking.

- **Redis:**  
  Maintains state and progress information for file and model processing.

- **RabbitMQ:**  
  Facilitates job dispatching between the API and Extractor services.

- **Ollama & Other Engines:**  
  Provides advanced processing capabilities by serving locally hosted language models. The system is extensible to support additional extraction or processing engines in the future.

---

## API Endpoints

### Welcome

```http
GET /
```

Returns a welcome message:
```
PDFz server is life 🔥🔥
```

---

### Web Interface

```http
GET /web
```
- Shows the web interface 

---

### Upload a File

```http
POST /upload
```

**Request:** Multipart form-data containing a `pdf` file.

**Response Example:**

```json
{
  "success": true,
  "message": "File uploaded successfully",
  "data": {
    "id": "file-id",
    "filename": "file.pdf",
    "path": "/shared_storage/upload/pdf/file.pdf",
    "size": 12345
  }
}
```

---

### Process a File

```http
POST /process/:id
```

**Request:** JSON body with processing options:
- `startPage` (default: 1)
- `pageCount` (default: 0)
- `priority` (default: 1)
- `engine` — extraction engine (e.g., `"tesseract"` or `"ollama"`)
- `model` — required if the selected engine is model-based (e.g., `"ollama"`)

Examples:

Using Tesseract:
```json
{
  "startPage": 1,
  "pageCount": 10,
  "priority": 1,
  "engine": "tesseract"
}
```

Using Ollama:
```json
{
  "startPage": 1,
  "pageCount": 10,
  "priority": 1,
  "engine": "ollama",
  "model": "llama3.2-vision"  // ":latest" will be appended if no tag is provided
}
```

**Response Example:**

```json
{
  "success": true,
  "message": "File processing started",
  "data": {
    "id": "file-id",
    "file": "file.pdf",
    "options": {
      "startPage": 1,
      "pageCount": 10,
      "priority": 1
    },
    "status": "queued",
    "progress": 0,
    "queuedAt": "2023-10-01T12:00:00Z"
  }
}
```

---

### Track File Processing Progress

```http
GET /progress/:id
```

**Response Example:**

```json
{
  "success": true,
  "message": "Progress retrieved successfully",
  "data": {
    "id": "file-id",
    "progress": 50,
    "status": "processing"
  }
}
```

---

### Retrieve Processed Content

```http
GET /content/:id
```

**Response Example:**

```json
{
  "success": true,
  "message": "Processed content retrieved successfully",
  "data": {
    "id": "file-id",
    "content": [
      {
        "page_num": 1,
        "text": "Text from page 1."
      },
      {
        "page_num": 2,
        "text": "Text from page 2."
      }
    ],
    "status": "completed"
  }
}
```

---

### Pull a Model (for Model-Based Extraction)

```http
POST /model/pull
```

**Request:** JSON body with the model name:

```json
{
  "model": "model-name"
}
```

**Response Examples:**

- **If the model already exists:**

  ```json
  {
    "success": true,
    "message": "Model already exists locally",
    "model": "model-name",
    "status": "exists"
  }
  ```

- **If the model is queued for download:**

  ```json
  {
    "success": true,
    "message": "Model download queued successfully",
    "model": "model-name",
    "status": "queued",
    "progress": 0
  }
  ```

---

### Track Model Download Progress

```http
GET /model/progress/:name
```

**Response Example:**

```json
{
  "success": true,
  "message": "Model progress retrieved successfully",
  "data": {
    "name": "model-name",
    "progress": 75,
    "status": "downloading"
  }
}
```

---

### List Available Models

```http
GET /models
```

**Response Example:**

```json
{
  "success": true,
  "message": "Models retrieved successfully",
  "data": {
    "models": [
      {
        "name": "model1:latest",
        "size": "1.2GB",
        "modified_at": "2023-10-01T12:00:00Z"
      },
      {
        "name": "model2:latest",
        "size": "900MB",
        "modified_at": "2023-09-28T08:30:00Z"
      }
    ]
  }
}
```

---

## Setup

### Prerequisites

#### For Docker Deployment:
- Docker & Docker Compose

#### For Local Development:

**API Service (Node.js & Express):**
- Node.js & npm
- Redis
- RabbitMQ

**Extractor Service (Rust):**
- Rust & Cargo
- Redis
- RabbitMQ
- At least one extraction engine (e.g., Tesseract OCR or an alternative)

**Ollama Service (for model-based extraction):**
- Docker container (or a local installation of Ollama)

---

### Installation

1. **Clone the Repository:**

   ```sh
   git clone https://github.com/codad5/pdfz.git
   cd pdfz
   ```

2. **Create an `.env` File:**

   ```sh
   cp .env.example .env
   ```

3. **Update Environment Variables:**  
   Modify the `.env` file to set your ports, RabbitMQ and Redis credentials, and extraction/model settings.

4. **Build and Start the Services:**

   ```sh
   docker-compose up --build
   ```

---

## Services & Environment Variables

### Extractor Service (Rust)

- `RUST_LOG=debug`  
- `REDIS_URL` — Redis connection URL  
- `RABBITMQ_URL` — RabbitMQ connection URL (e.g., `amqp://user:pass@rabbitmq:5672`)  
- `EXTRACTOR_PORT` — Port for the Extractor Service  
- `SHARED_STORAGE_PATH` — Mount point for file storage  
- `TRAINING_DATA_PATH` — Path to training data for extraction engines  
- `OLLAMA_BASE_URL` — Base URL for Ollama (e.g., `http://ollama:11434`)  
- `OLLAMA_BASE_PORT` — Ollama port (e.g., `11434`)  
- `OLLAMA_BASE_HOST` — Host for Ollama

### API Service (Node.js)

- `NODE_ENV=development`  
- `REDIS_URL` — Redis connection URL  
- `RABBITMQ_URL` — RabbitMQ connection URL  
- `API_PORT` — Port for the API service  
- `SHARED_STORAGE_PATH` — Mount point for file storage  
- `RABBITMQ_EXTRACTOR_QUEUE` — Queue name for file extraction requests  
- `OLLAMA_BASE_URL` — Base URL for Ollama  
- `OLLAMA_BASE_PORT` — Ollama port  
- `OLLAMA_BASE_HOST` — Host for Ollama

---

## Docker Compose Setup

Check the `docker-compose.yml` file to see the defined  services:


---

## Repository

For more details, visit the [GitHub repository](https://github.com/codad5/pdfz).

---

## Contributing

1. Fork the repository and create a new branch.  
2. Make changes and test locally.  
3. Submit a pull request.

---

## License

MIT License
