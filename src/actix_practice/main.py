from fastapi import FastAPI
import uvicorn

app = FastAPI()

@app.get("/python-fallback")
def read_root():
    return {"message": "Hello, World! from Python"}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)