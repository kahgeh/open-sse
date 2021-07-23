FROM node:12 as builder
WORKDIR /src
COPY *.json ./
COPY index.html ./
RUN npm i
COPY *.ts* ./
COPY *.js ./
RUN npm run build

EXPOSE 80
CMD ["node", "server.js"]