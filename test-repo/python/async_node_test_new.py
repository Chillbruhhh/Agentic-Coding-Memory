import asyncio

async def ping(name: str) -> str:
    await asyncio.sleep(0.01)
    return f"pong {name}"

if __name__ == "__main__":
    print(asyncio.run(ping("test")))
