import os
import time
import random
import requests
import pytest

BASE = "http://localhost:3742/api"
URL_GLOBAL = f"{BASE}/v1/global"
URL_STATIC = f"{BASE}/v1/static"

ASSETS_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")
# Matches OBJECT_STORE_PATH=itests/temp/storage in backend/.env
# (backend runs from the backend/ directory, so path is relative to it)
STORAGE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "temp", "storage")

ASSET_JPG = os.path.join(ASSETS_DIR, "photo_2025-09-13_00-46-10.jpg")
# Find the PNG asset by extension rather than hardcoding its Cyrillic filename.
_png_candidates = [f for f in os.listdir(ASSETS_DIR) if f.lower().endswith(".png")]
ASSET_PNG = os.path.join(ASSETS_DIR, _png_candidates[0]) if _png_candidates else None


def auth_headers(token: str) -> dict:
    return {"Authorization": f"Bearer {token}"}


def _wait_for_static(path: str, timeout: float = 10.0) -> bool:
    """Poll the static endpoint until the file is available or timeout expires."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        resp = requests.get(f"{URL_STATIC}/{path}")
        if resp.status_code == 200:
            return True
        time.sleep(0.3)
    return False


def _cleanup_avatar(ulid: str) -> None:
    """Remove processed avatar WebP files from local object store after a test."""
    for suffix in ("_hd.webp", "_thumb.webp"):
        path = os.path.join(STORAGE_DIR, "user_avatars", f"{ulid}{suffix}")
        try:
            os.remove(path)
        except FileNotFoundError:
            pass


def _upload_avatar(user_id: str, token: str, asset_path: str = None) -> requests.Response:
    asset_path = asset_path or ASSET_JPG
    with open(asset_path, "rb") as f:
        return requests.post(
            f"{URL_GLOBAL}/users/{user_id}/upload/avatar",
            headers=auth_headers(token),
            files={"file": (os.path.basename(asset_path), f, "image/jpeg")},
        )


# ------------------- Fixtures --------------------


@pytest.fixture(scope="module")
def admin_token():
    resp = requests.post(f"{BASE}/login", json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_user():
    num = random.randint(100000, 999999)
    user = f"avatar_user_{num}"
    requests.post(f"{BASE}/register", json={"user": user, "password": user})
    resp = requests.post(f"{BASE}/login", json={"user": user, "password": user})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {"user_id": f"u_{user}", "token": resp.json()["token"]}


@pytest.fixture(scope="module")
def other_user():
    num = random.randint(100000, 999999)
    user = f"avatar_other_{num}"
    requests.post(f"{BASE}/register", json={"user": user, "password": user})
    resp = requests.post(f"{BASE}/login", json={"user": user, "password": user})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {"user_id": f"u_{user}", "token": resp.json()["token"]}


# ------------------- Happy path: self-upload --------------------


def test_user_can_upload_own_avatar_jpg(regular_user):
    """User uploads a JPEG avatar for themselves; response contains a ulid."""
    resp = _upload_avatar(regular_user["user_id"], regular_user["token"])

    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"
    data = resp.json()
    assert "ulid" in data, f"Response missing 'ulid': {data}"
    assert len(data["ulid"]) > 0

    _cleanup_avatar(data["ulid"])


def test_upload_immediately_sets_avatar_ulid_on_user(regular_user):
    """avatar_ulid is visible on the user document right after the upload response."""
    resp = _upload_avatar(regular_user["user_id"], regular_user["token"])
    assert resp.status_code == 201
    ulid = resp.json()["ulid"]

    user_resp = requests.get(
        f"{URL_GLOBAL}/users/{regular_user['user_id']}",
        headers=auth_headers(regular_user["token"]),
    )
    assert user_resp.status_code == 200
    assert user_resp.json().get("avatar_ulid") == ulid, (
        f"Expected avatar_ulid={ulid!r}, got {user_resp.json().get('avatar_ulid')!r}"
    )

    _cleanup_avatar(ulid)


def test_user_can_upload_own_avatar_png(regular_user):
    """User uploads a PNG; endpoint accepts it and returns a ulid."""
    if ASSET_PNG is None:
        pytest.skip("no PNG asset found in assets/")
    with open(ASSET_PNG, "rb") as f:
        resp = requests.post(
            f"{URL_GLOBAL}/users/{regular_user['user_id']}/upload/avatar",
            headers=auth_headers(regular_user["token"]),
            files={"file": ("screenshot.png", f, "image/png")},
        )

    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"
    _cleanup_avatar(resp.json()["ulid"])


# ------------------- Static file serving --------------------


def test_processed_avatar_served_after_background_conversion(regular_user):
    """After background processing, HD and thumb WebP files appear on the static endpoint."""
    resp = _upload_avatar(regular_user["user_id"], regular_user["token"])
    assert resp.status_code == 201
    ulid = resp.json()["ulid"]

    hd_path = f"user_avatars/{ulid}_hd.webp"
    thumb_path = f"user_avatars/{ulid}_thumb.webp"

    assert _wait_for_static(hd_path), (
        f"HD avatar not available within 10 s: GET /v1/static/{hd_path}"
    )
    assert _wait_for_static(thumb_path), (
        f"Thumbnail not available within 10 s: GET /v1/static/{thumb_path}"
    )

    hd_resp = requests.get(f"{URL_STATIC}/{hd_path}")
    assert hd_resp.status_code == 200
    assert hd_resp.headers.get("Content-Type") == "image/webp"
    assert "public" in hd_resp.headers.get("Cache-Control", ""), (
        f"Expected Cache-Control: public, got: {hd_resp.headers.get('Cache-Control')}"
    )
    assert len(hd_resp.content) > 0, "HD file body is empty"

    _cleanup_avatar(ulid)


def test_static_avatar_accessible_without_auth(regular_user):
    """Processed avatar files are served without a Bearer token."""
    resp = _upload_avatar(regular_user["user_id"], regular_user["token"])
    assert resp.status_code == 201
    ulid = resp.json()["ulid"]

    hd_path = f"user_avatars/{ulid}_hd.webp"
    _wait_for_static(hd_path)

    # No Authorization header
    resp = requests.get(f"{URL_STATIC}/{hd_path}")
    assert resp.status_code == 200, (
        f"Expected 200 without auth, got {resp.status_code}: {resp.text}"
    )

    _cleanup_avatar(ulid)


# ------------------- Admin upload --------------------


def test_admin_can_upload_avatar_for_another_user(admin_token, regular_user):
    """Root (ADM_GODMODE) can upload an avatar on behalf of any user."""
    resp = _upload_avatar(regular_user["user_id"], admin_token)
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"
    _cleanup_avatar(resp.json()["ulid"])


# ------------------- Access control --------------------


def test_other_user_cannot_upload_avatar_for_target(regular_user, other_user):
    """A regular user cannot upload an avatar for a different user; gets 404."""
    resp = _upload_avatar(regular_user["user_id"], other_user["token"])
    assert resp.status_code == 404, (
        f"Expected 404 (ACL denied), got {resp.status_code}: {resp.text}"
    )


def test_unauthenticated_upload_rejected(regular_user):
    """Upload without a Bearer token returns 401."""
    with open(ASSET_JPG, "rb") as f:
        resp = requests.post(
            f"{URL_GLOBAL}/users/{regular_user['user_id']}/upload/avatar",
            files={"file": ("photo.jpg", f, "image/jpeg")},
        )
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


# ------------------- Input validation --------------------


def test_invalid_upload_type_rejected(regular_user):
    """An unknown upload_type returns 400."""
    with open(ASSET_JPG, "rb") as f:
        resp = requests.post(
            f"{URL_GLOBAL}/users/{regular_user['user_id']}/upload/banner",
            headers=auth_headers(regular_user["token"]),
            files={"file": ("photo.jpg", f, "image/jpeg")},
        )
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


def test_missing_file_field_rejected(regular_user):
    """Multipart body without a 'file' field returns 400."""
    resp = requests.post(
        f"{URL_GLOBAL}/users/{regular_user['user_id']}/upload/avatar",
        headers=auth_headers(regular_user["token"]),
        files={"other_field": ("photo.jpg", b"some bytes", "image/jpeg")},
    )
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


def test_non_image_bytes_rejected(regular_user):
    """Uploading a file whose bytes are not a valid image format returns 400."""
    resp = requests.post(
        f"{URL_GLOBAL}/users/{regular_user['user_id']}/upload/avatar",
        headers=auth_headers(regular_user["token"]),
        files={"file": ("data.bin", b"this is not an image", "application/octet-stream")},
    )
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


def test_upload_for_nonexistent_user_returns_404(admin_token):
    """Uploading for a user ID that does not exist returns 404."""
    with open(ASSET_JPG, "rb") as f:
        resp = requests.post(
            f"{URL_GLOBAL}/users/u_no_such_user_xyzzy/upload/avatar",
            headers=auth_headers(admin_token),
            files={"file": ("photo.jpg", f, "image/jpeg")},
        )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"


# ------------------- Static file security --------------------


def test_static_rejects_paths_outside_allowed_dirs():
    """Paths not under user_avatars/ or user_wallpapers/ return 404."""
    for bad_path in ("raw_uploads/something.jpg", "etc/passwd", "secrets"):
        resp = requests.get(f"{URL_STATIC}/{bad_path}")
        assert resp.status_code == 404, (
            f"Expected 404 for path {bad_path!r}, got {resp.status_code}"
        )


def test_static_rejects_path_traversal():
    """Paths containing '..' return 404."""
    for traversal in (
        "user_avatars/../raw_uploads/file.jpg",
        "user_avatars/../../etc/passwd",
    ):
        resp = requests.get(f"{URL_STATIC}/{traversal}")
        assert resp.status_code == 404, (
            f"Expected 404 for traversal {traversal!r}, got {resp.status_code}"
        )


def test_static_returns_404_for_nonexistent_avatar():
    """A well-formed path to a ULID that was never uploaded returns 404."""
    fake_path = "user_avatars/01jz000000000000000000000_hd.webp"
    resp = requests.get(f"{URL_STATIC}/{fake_path}")
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"
