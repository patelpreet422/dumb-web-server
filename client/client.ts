import axios, { AxiosResponse } from 'axios';

const response = await axios.get('http://localhost:8080', { responseType: 'stream' });

const stream = response.data;

stream.on('data', (data: any) => {
    console.log(Buffer.from(data).toString());
});

stream.on('end', () => {
    console.log("stream done");
});