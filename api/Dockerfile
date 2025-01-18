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

