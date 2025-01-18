# Base image
FROM node:18-alpine

# Set the working directory
WORKDIR /app

# Copy package.json and install dependencies
COPY package*.json ./
RUN npm install

# Install nodemon globally
RUN npm install -g nodemon

# Copy application code
COPY . .

# Default command for development with nodemon
CMD ["npm", "run", "dev"]



# FROM node:20-slim

# # Install protobuf compiler
# RUN apt-get update && \
#     apt-get install -y protobuf-compiler && \
#     apt-get clean && \
#     rm -rf /var/lib/apt/lists/*

# WORKDIR /app

# # Copy package files
# COPY package*.json ./

# # Install dependencies
# RUN npm install

# # Copy proto files
# # COPY proto/ ./proto/

# # Copy the rest of the application
# COPY . .

# EXPOSE 3000
# CMD ["npm", "start"]