# rodan-noitifications

A **Server-Sent Events (SSE)** microservice for Rodan, designed to support **notifications**.

## Features / Work Done

* Fully implemented and production-ready SSE service.  
* Events are treated as **fire-and-forget**; no WAL or persistent storage is used.  
* Uses `tokio::sync::broadcast` for efficient **real-time notifications**.  

## Integration

* Integrated with **rodan-core** and **rodan-admin** to ensure compatibility and correctness.  

