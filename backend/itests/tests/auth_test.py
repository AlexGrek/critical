import pytest
import requests
import json
import re
import random
import websockets
import asyncio


URL_LOGIN = "http://localhost:3742/api/login"
URL_REGISTER = "http://localhost:3742/api/register"
URL_WS = "ws://localhost:3742/api/v1/ws"


def is_jwt(s: str) -> bool:
    return bool(
        re.fullmatch(r"^[A-Za-z0-9_\-]+=*\.[A-Za-z0-9_\-]+=*\.[A-Za-z0-9_\-]+=?$", s)
    )


class Main:
    token: str

    def __init__(self, token: str) -> None:
        self.token = token


class Error:
    message: str
    status: int
    type: str

    def __init__(self, message: str, status: int, type: str) -> None:
        self.message = message
        self.status = status
        self.type = type


class MainErr:
    error: Error

    def __init__(self, error: Error) -> None:
        self.error = error


@pytest.fixture(scope="module")
def test_user():
    """Create a random test user credentials"""
    num = random.randint(100000, 999999)
    return {
        "username": f"itest{num}",
        "password": f"itest{num}"
    }


@pytest.fixture(scope="module")
def auth_token(test_user):
    """Login and get JWT token for the test user"""
    headers = {
        "Accept": "*/*",
        "User-Agent": "pytest-client",
        "Content-Type": "application/json",
    }
    payload = {"user": test_user["username"], "password": test_user["password"]}

    resp = requests.post(URL_LOGIN, json=payload, headers=headers)
    assert resp.status_code == 200, f"Login failed: {resp.text}"

    data = resp.json()
    assert "token" in data, "No token in response"
    
    return data["token"]


def test_1_register_user(test_user):
    """Test 1: Register a new user"""
    headers = {
        "Accept": "*/*",
        "User-Agent": "pytest-client",
        "Content-Type": "application/json",
    }

    payload = {
        "user": test_user["username"],
        "password": test_user["password"],
    }

    resp = requests.post(URL_REGISTER, json=payload, headers=headers)
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"


def test_2_login_success(test_user):
    """Test 2: Login with the registered user"""
    headers = {
        "Accept": "*/*",
        "User-Agent": "pytest-client",
        "Content-Type": "application/json",
    }
    payload = {"user": test_user["username"], "password": test_user["password"]}

    resp = requests.post(URL_LOGIN, json=payload, headers=headers)
    assert resp.status_code == 200, resp.text

    data = resp.json()
    assert "token" in data

    main = Main(token=data["token"])
    assert isinstance(main.token, str)
    assert main.token != ""
    assert is_jwt(main.token)


def test_3_login_failure():
    """Test 3: Login with invalid credentials"""
    headers = {
        "Accept": "*/*",
        "User-Agent": "pytest-client",
        "Content-Type": "application/json",
    }
    payload = {"user": "wronguser", "password": "totallywrong"}

    resp = requests.post(URL_LOGIN, json=payload, headers=headers)
    assert resp.status_code == 401, resp.text

    data = resp.json()
    assert "error" in data

    e = data["error"]
    main_err = MainErr(Error(e["message"], e["status"], e["type"]))

    assert main_err.error.message == "Authorization failed: Unauthorized"
    assert main_err.error.status == 401
    assert main_err.error.type == "authorization_error"


def test_4_websocket_connection(auth_token):
    """Test 4: Establish WebSocket connection with JWT token"""
    
    async def test_ws():
        headers = [
            ("Authorization", f"Bearer {auth_token}")
        ]
        
        async with websockets.connect(URL_WS, additional_headers=headers) as ws:
            # Connection is established if we reach here without exception
            # Optional: Send a test message and receive response
            # await ws.send(json.dumps({"type": "ping"}))
            # response = await ws.recv()
            # assert response is not None
            pass
    
    # Run the async test
    asyncio.run(test_ws())
