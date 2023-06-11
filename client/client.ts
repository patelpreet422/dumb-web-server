import axios, { AxiosResponse } from 'axios';
import EventSource from 'eventsource';
import { url } from 'inspector';

const transferEncodingChunked = async () => {
    const response = await axios.get('http://localhost:8080', { responseType: 'stream' });

    const stream = response.data;

    stream.on('data', (data: any) => {
        console.log(Buffer.from(data).toString());
    });

    stream.on('end', () => {
        console.log("stream done");
    });
}

const sse = async () => {
    const eventSource = new EventSource('http://localhost:8080');

    eventSource.onopen = () => {
      console.log('SSE connection opened');
    };
    
    eventSource.onmessage = (event) => {
      const eventData = event.data;
      console.log('Received SSE message:', eventData);
    };
    
    eventSource.onerror = (error) => {
      console.error('SSE error:', error);
    };
}

await transferEncodingChunked()
