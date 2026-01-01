import asyncio
from fastapi import FastAPI, Query
from fastapi.responses import StreamingResponse
from pytz import timezone
import uvicorn
from datetime import datetime
from fastapi import Request

app = FastAPI()

@app.get("/python-fallback")
def read_root():
    return {"message": "Hello, World! from Python"}

@app.get("/python-stream-time")
async def stream_time(timezone_str: str = Query(default="UTC")):
    async def _stream_time():
        while True:
            yield f"data: {datetime.now(timezone(timezone_str)).isoformat()}\n\n"
            await asyncio.sleep(1)  
    return StreamingResponse(_stream_time(), media_type="text/event-stream")

@app.post("/api/hey")
async def hey(request: Request):
    body = await request.json()
    name = body.get("name")
    return {"message": f"Hello, {name}! from Python"}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)