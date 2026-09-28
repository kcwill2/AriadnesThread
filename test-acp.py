import json
import subprocess
import sys
import threading

EXE = r"C:\Users\Kurt\AppData\Roaming\npm\node_modules\@opencode\cli\bin\opencode.exe"
CWD = r"C:\Users\Kurt\Code\AriadnesThread"

p = subprocess.Popen(
    [EXE, "acp"],
    cwd=CWD,
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    bufsize=1,
)


def read_stderr():
    for line in p.stderr:
        print(f"STDERR: {line}", end="", file=sys.stderr)


threading.Thread(target=read_stderr, daemon=True).start()


def send(obj):
    data = json.dumps(obj, separators=(",", ":"))
    print(f">>> {data}")
    p.stdin.write(data + "\n")
    p.stdin.flush()


def recv():
    line = p.stdout.readline()

    if not line:
        print("<<< EOF")
        print(f"Process exited with code {p.poll()}")
        sys.exit(1)

    print(f"<<< {line.rstrip()}")
    return json.loads(line)


def recv_response(request_id):
    """Read messages until we receive the response to our request."""
    while True:
        msg = recv()

        # JSON-RPC response to our request
        if msg.get("id") == request_id:
            return msg

        # ACP notification
        print(f"... notification/update received")


# 1. Initialize
send({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": 1,
        "clientCapabilities": {},
        "clientInfo": {
            "name": "manual-acp-test",
            "title": "Manual ACP Test",
            "version": "1.0.0"
        }
    }
})

init = recv_response(1)

print()
print("=== INITIALIZE RESULT ===")
print(json.dumps(init, indent=2))
print()


# 2. Create session
send({
    "jsonrpc": "2.0",
    "id": 2,
    "method": "session/new",
    "params": {
        "cwd": CWD,
        "mcpServers": []
    }
})

session_response = recv_response(2)

print()
print("=== SESSION/NEW RESULT ===")
print(json.dumps(session_response, indent=2))
print()

session_id = session_response["result"]["sessionId"]

print(f"SESSION ID: {session_id}")
print()


# 3. Send a trivial prompt
send({
    "jsonrpc": "2.0",
    "id": 3,
    "method": "session/prompt",
    "params": {
        "sessionId": session_id,
        "prompt": [
            {
                "type": "text",
                "text": "Say hello."
            }
        ]
    }
})


# 4. Read everything until the prompt request completes
while True:
    msg = recv()

    if msg.get("id") == 3:
        print()
        print("=== PROMPT RESPONSE ===")
        print(json.dumps(msg, indent=2))
        break